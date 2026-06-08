use std::fs::File;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use biobit_core_rs::loc::{Interval, Orientation};
use biobit_core_rs::parallelism;
use biobit_core_rs::source::Source;
use biobit_io_rs::{
    bam::{ReaderBuilder, strdeductor, transform},
    fasta,
};
use biobit_reat_rs::dna;
use biobit_reat_rs::selection::Selector;
use biobit_reat_rs::{
    SamplePileup,
    selection::{Mismatches, RequiredOrMismatches, RequiredSites},
    task::Task,
};
use eyre::{Result, ensure, eyre};
use itertools::Itertools;
use rayon::ThreadPoolBuilder;
use substratum_compress::{
    Decoder, Encoder, adapter::BoxedSync, decode::DecodeReadIntoBufRead, encode::Encode,
};

const THREADS: isize = -1;
const MIN_PHRED: u8 = 20;
const SAMPLE: &str = "PE-reverse";
const REMAKE_EXPECTED_ENV: &str = "BIOBIT_REAT_REMAKE_EXPECTED";
const MAX_TASK_SIZE: u64 = 64_000;

const TASK_INTERVALS: &[(&str, u64, u64)] = &[
    ("chr21", 11_052_649, 11_054_065),
    ("chr21", 30_562_496, 31_269_216),
    ("chr21", 37_206_181, 37_209_212),
    ("chr22", 10_982_293, 11_045_159),
    ("chr22", 11_012_207, 11_012_523),
    ("chr22", 24_632_672, 24_640_676),
    ("chr22", 25_785_074, 25_962_399),
    ("chr22", 29_574_184, 33_045_489),
    ("chr22", 40_511_802, 40_589_004),
    ("chr22", 46_328_399, 46_328_898),
];

const REQUIRED_INTERVALS: &[(&str, u64, u64)] = &[
    ("chr21", 37_207_315, 37_207_320),
    ("chr22", 46_328_522, 46_328_525),
];

#[derive(Clone, PartialEq, Eq, Debug)]
struct Row {
    seqid: String,
    orientation: Orientation,
    position: u64,
    reference: dna::Reference,
    a: u32,
    c: u32,
    g: u32,
    t: u32,
    n: u32,
    deletion: u32,
}

impl Row {
    fn from_str(line: &str, sep: char) -> Result<Self> {
        let (seqid, orientation, position, reference, a, c, g, t, n, deletion) = line
            .split(sep)
            .collect_tuple()
            .ok_or_else(|| eyre!("Invalid Row string: {}", line))?;

        let orientation = Orientation::try_from(orientation)
            .map_err(|_| eyre!("Invalid orientation in row: {}", line))?;

        Ok(Row {
            seqid: seqid.to_string(),
            orientation,
            position: position.parse()?,
            reference: dna::Reference::try_from(reference)?,
            a: a.parse()?,
            c: c.parse()?,
            g: g.parse()?,
            t: t.parse()?,
            n: n.parse()?,
            deletion: deletion.parse()?,
        })
    }
}

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

fn selector() -> Result<Arc<dyn Selector<String, u64, u32> + Send + Sync + 'static>> {
    let mut required = Vec::new();
    for (seqid, start, end) in REQUIRED_INTERVALS {
        let interval = Interval::new(*start, *end)?;
        for orientation in [Orientation::Forward, Orientation::Reverse] {
            required.push((seqid.to_string(), orientation, vec![interval]));
        }
    }
    let required = RequiredSites::new(required);
    let mismatches = Mismatches::new(4, 0.05)?;
    Ok(Arc::new(RequiredOrMismatches::new(required, mismatches)))
}

#[test]
fn regression() -> Result<()> {
    let rows = run()?;
    ensure!(!rows.is_empty(), "REAT regression produced no rows");

    let expected = get_resource_path("regression-tests/expected.csv.gz")?;
    if std::env::var(REMAKE_EXPECTED_ENV).unwrap_or("0".into()) == "1" {
        write_expected(&expected, &rows)?;
    }

    compare_expected(&expected, &rows)
}

fn run() -> Result<Vec<Row>> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(parallelism::available(THREADS)?)
        .use_current_thread()
        .build()?;

    let reference = get_resource_path("regression-tests/reference.fa.bgz")?;
    let reference = biobit_io_rs::fasta::IndexedSources::from_path(
        &reference,
        Decoder::from_path(&reference, fasta::EXTENSIONS)?,
    );

    let mut engine = biobit_reat_rs::Reat::new(pool, reference, MIN_PHRED, selector()?);

    let bam = get_resource_path("regression-tests/input.bam")?;
    let source = ReaderBuilder::new(&bam)
        .with_inflags(3)
        .with_exflags(2572)
        .with_minmapq(0)
        .build()?
        .with_transform(
            transform::BundleByOrientation::new(strdeductor::deduce::pe::reverse),
            (),
        );
    engine.register(SAMPLE.to_string(), [source]);

    let intervals = TASK_INTERVALS
        .iter()
        .map(|(seqid, start, end)| Ok((seqid.to_string(), Interval::new(*start, *end)?)))
        .collect::<Result<Vec<_>>>()?;
    let tasks = Task::from_intervals(intervals, MAX_TASK_SIZE)?;

    let result = engine.run(tasks)?;
    ensure!(
        result.len() == 1,
        "Expected one sample, found {}",
        result.len()
    );
    ensure!(
        result[0].tag == SAMPLE,
        "Unexpected sample tag: {}",
        result[0].tag
    );

    Ok(flatten(&result[0]))
}

fn flatten(result: &SamplePileup<String, u64, u32, String>) -> Vec<Row> {
    let mut rows = Vec::new();
    for ((seqid, orientation), pileup) in &result.pileups {
        debug_assert_eq!(pileup.pileup().len(), pileup.reference().len());
        for ((position, counts), reference) in pileup.pileup().iter().zip(pileup.reference()) {
            rows.push(Row {
                seqid: seqid.clone(),
                orientation: *orientation,
                position,
                reference: *reference,
                a: *counts.a(),
                c: *counts.c(),
                g: *counts.g(),
                t: *counts.t(),
                n: *counts.n(),
                deletion: *counts.deletion(),
            });
        }
    }

    rows.sort_by(|left, right| {
        (left.seqid.as_str(), left.orientation, left.position).cmp(&(
            right.seqid.as_str(),
            right.orientation,
            right.position,
        ))
    });
    rows
}

fn write_expected(path: &Path, rows: &[Row]) -> Result<()> {
    let encoder = Encoder::from_path(path, &["csv"])?;
    let mut writer = encoder.encode(File::create(path)?, BoxedSync)?;
    writeln!(
        writer,
        "seqid,orientation,position,reference,A,C,G,T,N,deletion"
    )?;
    for row in rows {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{},{}",
            row.seqid,
            row.orientation,
            row.position,
            row.reference,
            row.a,
            row.c,
            row.g,
            row.t,
            row.n,
            row.deletion
        )?;
    }
    writer.flush()?;
    Ok(())
}

fn compare_expected(path: &Path, rows: &[Row]) -> Result<()> {
    ensure!(
        path.exists(),
        "Expected output does not exist: {}. Set {}=1 to regenerate it.",
        path.display(),
        REMAKE_EXPECTED_ENV
    );

    let decoder = Decoder::from_path(path, &["csv", "tsv"])?;
    let reader = decoder.decode_read_into_bufread(File::open(path)?, BoxedSync)?;
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or_else(|| eyre!("Expected output is empty: {}", path.display()))??;
    ensure!(
        header == "seqid,orientation,position,reference,A,C,G,T,N,deletion",
        "Unexpected expected-output header: {}",
        header
    );

    let mut row = 0;
    for line in lines {
        let line = line?;
        ensure!(
            row < rows.len(),
            "Expected output has more rows than current result; first extra row {}: {}",
            row,
            line
        );
        let expected = Row::from_str(&line, ',')?;
        ensure!(
            rows[row] == expected,
            "Mismatch at row {}: expected {:?}, found {:?}",
            row,
            expected,
            rows[row]
        );
        row += 1;
    }

    ensure!(
        row == rows.len(),
        "Expected {} rows, found {}",
        rows.len(),
        row
    );
    Ok(())
}
