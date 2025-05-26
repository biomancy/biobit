use ahash::{HashMap, HashMapExt};
use rayon::ThreadPoolBuilder;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::string::ToString;

use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::parallelism;
use biobit_core_rs::source::{DynSource, Source};
use biobit_io_rs::{
    bam::{ReaderBuilder, strdeductor, transform},
    bed,
    bed::{Bed3Op, Bed4Op, Bed6Op, Bed12Op},
    compression,
};
use eyre::{Result, ensure, eyre};
use itertools::Itertools;

const THREADS: isize = -1;
const EPSILON: f64 = 1e-6;
const PARTITIONS: &[(&str, usize, usize)] = &[("chr21", 0, 45090682), ("chr22", 0, 51324926)];

pub fn get_resource_path(resource: impl AsRef<Path>) -> Result<PathBuf> {
    let resource = resource.as_ref();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|x| x.join("resources"))
        .map(|x| x.join(resource))
        .ok_or_else(|| {
            eyre!(
                "Failed to locate requested resource: {}",
                resource.display()
            )
        })?;
    ensure!(
        path.exists(),
        "Requested resource does not exist: {}",
        path.display()
    );
    Ok(path)
}

#[test]
fn regression() -> Result<()> {
    let mut builder = biobit_countit_rs::rigid::Engine::<String, usize, f64, String>::builder();

    // Specify the thread pool
    let pool = ThreadPoolBuilder::new()
        .num_threads(parallelism::available(THREADS)?)
        .use_current_thread()
        .build()?;
    builder = builder.set_thread_pool(pool);

    // Add regions that will be fetched and processed
    builder =
        builder.add_partitions(PARTITIONS.iter().map(|(contig, start, end)| {
            (contig.to_string(), Interval::new(*start, *end).unwrap())
        }));

    // Define the target annotation
    let bed = get_resource_path("regression-tests/annotation.bed.bgz")?;
    let compression = compression::decode::Config::infer_from_path(&bed);

    let mut records = Vec::new();
    let mut reader = bed::Reader::from_path::<bed::Bed12>(&bed, &compression)?;
    reader.read_to_end(&mut records)?;

    // Define counting elements from BED12 records. Preserve the order of names.
    let mut elements = HashMap::new();
    let mut names_order = HashMap::new();
    for record in records.into_iter() {
        if record.seqid() != "chr21" && record.seqid() != "chr22" && record.seqid() != "chrM" {
            continue; // Skip records outside the specified partitions and one extra chromosome
        }

        names_order
            .entry(record.name().to_string())
            .or_insert(elements.len());

        let entry = elements
            .entry(record.name().to_string())
            .or_insert_with(HashMap::new)
            .entry((record.seqid().to_string(), record.orientation()))
            .or_insert_with(Vec::new);

        let start = record.interval().start();
        for block in record.blocks() {
            entry.push(block.shifted(start).cast::<usize>().unwrap())
        }
    }
    let elements = elements
        .into_iter()
        .sorted_by_key(|x| names_order.get(&x.0).unwrap());

    builder = builder.add_elements(elements.into_iter().map(|(name, items)| {
        let items = items
            .into_iter()
            .map(|((name, orientation), intervals)| (name, orientation, intervals))
            .collect();
        (name, items)
    }));

    // Finalize the engine
    let mut engine = builder.build::<f64>();

    // Define the alignment source. Treating a single BAM file as a source of PE and SE reads.
    let path = get_resource_path("regression-tests/input.bam")?;
    let pe = ReaderBuilder::new(&path)
        .with_inflags(3)
        .with_exflags(2572)
        .with_minmapq(0)
        .build()?
        .with_transform(transform::BundleMates::default(), ())
        .with_transform(
            transform::ExtractPairedAlignmentSegments::new(strdeductor::deduce::pe::reverse),
            (),
        )
        .to_dynsrc()
        .to_src()
        .boxed();
    let se = ReaderBuilder::new(&path)
        .with_minmapq(255)
        .build()?
        .with_transform(
            transform::ExtractAlignmentSegments::new(strdeductor::deduce::se::forward),
            (),
        )
        .to_dynsrc()
        .to_src()
        .boxed();
    let sources = vec![("PE-reads".to_string(), pe), ("SE-reads".to_string(), se)];

    // Define the resolution strategy
    let resolution = Box::new(biobit_countit_rs::rigid::resolution::OverlapWeighted::new());

    // Run the countit
    let result = engine.run(sources.into_iter(), resolution)?;
    assert_eq!(result.len(), 2);
    for r in &result {
        ensure!(
            r.partitions.len() == PARTITIONS.len(),
            "Unexpected number of partitions in {}",
            r.source
        );
        ensure!(
            r.elements.len() > 0,
            "No elements found in source: {}",
            r.source
        );
        ensure!(
            r.counts.len() == r.elements.len(),
            "Counts length mismatch in source: {}",
            r.source
        );
    }

    let (mut pe, mut se) = (&result[0], &result[1]);
    if pe.source != "PE-reads" {
        std::mem::swap(&mut pe, &mut se);
    }
    ensure!(
        std::ptr::addr_eq(pe.elements, se.elements),
        "References to element array in PE and SE sources do not match"
    );
    let expnames = names_order
        .into_iter()
        .sorted_by_key(|x| x.1)
        .map(|x| x.0)
        .collect_vec();
    ensure!(
        se.elements == &expnames,
        "Elements in the result do not match expected names"
    );

    let output = get_resource_path("regression-tests/expected.csv.gz")?;
    // Uncomment to regenerate the "ground truth" for the regression test.
    // let mut writer = biobit_io_rs::compression::encode::infer_from_path(&output)?.boxed();
    // writeln!(writer, "element,PE-reads,SE-reads")?;
    // for (i, element) in pe.elements.iter().enumerate() {
    //     writeln!(
    //         writer,
    //         "{},{:.6},{:.6}",
    //         element, pe.counts[i], se.counts[i]
    //     )?;
    // }

    // Check the results against the expected output
    let mut lines = BufReader::new(compression::decode::infer_from_path(&output)?.boxed()).lines();
    assert_eq!(lines.next().unwrap()?, "element,PE-reads,SE-reads");

    let mut row = 0;
    for line in lines {
        let line = line?;
        let (element, pecnt, secnt) = line
            .split(',')
            .collect_tuple()
            .ok_or_else(|| eyre!("Invalid line: {}", line))?;
        let pecnt: f64 = pecnt.parse()?;
        let secnt: f64 = secnt.parse()?;

        ensure!(
            (element == &pe.elements[row])
                && (pe.counts[row] - pecnt).abs() <= EPSILON
                && (se.counts[row] - secnt).abs() <= EPSILON,
            "Mismatch at row {}: expected ({}, {:.6}, {:.6}), found ({}, {:.6}, {:.6})",
            row,
            pe.elements[row],
            pe.counts[row],
            se.counts[row],
            element,
            pecnt,
            secnt
        );
        row += 1;
    }
    ensure!(
        row == pe.elements.len(),
        "Expected {} rows, found {}",
        pe.elements.len(),
        row
    );

    Ok(())
}
