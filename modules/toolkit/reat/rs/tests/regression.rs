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
use biobit_reat_rs::{
    SelectedPileup,
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
const MAX_TASK_SIZE: u64 = 25_000;

const TASK_INTERVALS: &[(&str, u64, u64)] = &[
    ("chr21", 238_800, 238_900),
    ("chr21", 238_850, 238_950),
    ("chr21", 238_950, 239_050),
    ("chr21", 2_125_000, 2_205_000),
    ("chr22", 235_700, 235_800),
    ("chr22", 235_750, 235_850),
    ("chr22", 235_850, 235_950),
    ("chr22", 3_625_000, 3_705_000),
];

const REQUIRED_INTERVALS: &[(&str, u64, u64)] = &[
    ("chr21", 238_800, 238_801),
    ("chr21", 238_900, 238_901),
    ("chr21", 239_000, 239_001),
    ("chr21", 2_125_000, 2_125_001),
    ("chr22", 235_700, 235_701),
    ("chr22", 235_800, 235_801),
    ("chr22", 235_900, 235_901),
    ("chr22", 3_625_000, 3_625_001),
];

#[derive(Clone, PartialEq, Eq, Debug)]
struct Row {
    seqid: String,
    orientation: Orientation,
    position: u64,
    a: u32,
    c: u32,
    g: u32,
    t: u32,
    n: u32,
    deletion: u32,
}

pub fn resource_path(resource: impl AsRef<Path>) -> Result<PathBuf> {
    let resource = resource.as_ref();
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|x| x.join("resources"))
        .map(|x| x.join(resource))
        .ok_or_else(|| {
            eyre!(
                "Failed to locate requested resource: {}",
                resource.display()
            )
        })
}

pub fn get_resource_path(resource: impl AsRef<Path>) -> Result<PathBuf> {
    let resource = resource.as_ref();
    let path = resource_path(resource)?;
    ensure!(
        path.exists(),
        "Requested resource does not exist: {}",
        path.display()
    );
    Ok(path)
}

#[test]
fn regression() -> Result<()> {
    let rows = run_pipeline()?;
    ensure!(!rows.is_empty(), "REAT regression produced no rows");

    let expected = resource_path("regression-tests/expected.csv.gz")?;
    if remake_expected() {
        write_expected(&expected, &rows)?;
    }

    compare_expected(&expected, &rows)
}

fn run_pipeline() -> Result<Vec<Row>> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(parallelism::available(THREADS)?)
        .use_current_thread()
        .build()?;

    let reference = get_resource_path("regression-tests/reference.fa.bgz")?;
    let reference = biobit_io_rs::fasta::IndexedSources::from_path(
        &reference,
        Decoder::from_path(&reference, fasta::EXTENSIONS)?,
    );

    let required = required_sites()?;
    let mismatches = Mismatches::new(10_u32, 0.10, 101)?;
    let selector = Arc::new(RequiredOrMismatches::new(required, mismatches));

    let mut engine = biobit_reat_rs::Reat::new(pool, reference, MIN_PHRED, selector);

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

fn required_sites() -> Result<RequiredSites<String, u64>> {
    let mut required = Vec::new();
    for (seqid, start, end) in REQUIRED_INTERVALS {
        let interval = Interval::new(*start, *end)?;
        for orientation in [Orientation::Forward, Orientation::Reverse] {
            required.push((seqid.to_string(), orientation, vec![interval]));
        }
    }
    Ok(RequiredSites::new(required))
}

fn flatten(result: &SelectedPileup<String, u64, u32, String>) -> Vec<Row> {
    let mut rows = Vec::new();
    for ((seqid, orientation), pileup) in &result.pileups {
        for (position, counts) in pileup.iter() {
            rows.push(Row {
                seqid: seqid.clone(),
                orientation: *orientation,
                position,
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
        (
            left.seqid.as_str(),
            orientation_rank(left.orientation),
            left.position,
        )
            .cmp(&(
                right.seqid.as_str(),
                orientation_rank(right.orientation),
                right.position,
            ))
    });
    rows
}

fn orientation_rank(orientation: Orientation) -> u8 {
    match orientation {
        Orientation::Forward => 0,
        Orientation::Reverse => 1,
        Orientation::Dual => 2,
    }
}

fn remake_expected() -> bool {
    std::env::var(REMAKE_EXPECTED_ENV).is_ok_and(|value| {
        let value = value.to_ascii_lowercase();
        matches!(value.as_str(), "1" | "true" | "yes")
    })
}

fn write_expected(path: &Path, rows: &[Row]) -> Result<()> {
    let encoder = Encoder::from_path(path, &["csv", "tsv"])?;
    let mut writer = encoder.encode(File::create(path)?, BoxedSync)?;
    writeln!(writer, "seqid,orientation,position,A,C,G,T,N,deletion")?;
    for row in rows {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{}",
            row.seqid,
            row.orientation,
            row.position,
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
        header == "seqid,orientation,position,A,C,G,T,N,deletion",
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
        let expected = parse_row(&line)?;
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

fn parse_row(line: &str) -> Result<Row> {
    let (seqid, orientation, position, a, c, g, t, n, deletion) =
        line.split(',')
            .collect_tuple()
            .ok_or_else(|| eyre!("Invalid expected-output row: {}", line))?;

    let orientation = Orientation::try_from(orientation)
        .map_err(|_| eyre!("Invalid orientation in expected-output row: {}", line))?;

    Ok(Row {
        seqid: seqid.to_string(),
        orientation,
        position: position.parse()?,
        a: a.parse()?,
        c: c.parse()?,
        g: g.parse()?,
        t: t.parse()?,
        n: n.parse()?,
        deletion: deletion.parse()?,
    })
}
