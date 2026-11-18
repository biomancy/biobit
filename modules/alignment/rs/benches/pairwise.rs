use std::hint::black_box;
use std::time::Duration;

use biobit_alignment_rs::pairwise::{
    dp::{
        ChunkSize, Engine, Scan,
        layout::Layout,
        tracer::{AllOptimal, Best, TraceMatrix},
    },
    scoring,
};
use biobit_io_rs::{ReadRecord, fasta};
use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};

const CHUNK_SIZES: [ChunkSize; 7] = [
    ChunkSize::Rows1,
    ChunkSize::Rows16,
    ChunkSize::Rows64,
    ChunkSize::Rows128,
    ChunkSize::Rows256,
    ChunkSize::Rows512,
    ChunkSize::Rows1024,
];

fn strands(record: &fasta::Record) -> (Vec<u8>, Vec<u8>) {
    let bases = record.seq();
    // Keep the debug smoke run from cargo test --all-targets short.
    let bases = if cfg!(debug_assertions) {
        &bases[..bases.len().min(65)]
    } else {
        bases
    };
    let forward = bases.to_ascii_uppercase();
    let reverse = forward
        .iter()
        .rev()
        .map(|&base| match base {
            b'A' => b'T',
            b'C' => b'G',
            b'G' => b'C',
            b'T' => b'A',
            other => panic!(
                "Unexpected DNA base '{}' in {}",
                char::from(other),
                record.id()
            ),
        })
        .collect();
    (forward, reverse)
}

fn bench_scan(
    criterion: &mut Criterion,
    region: &str,
    layout: &str,
    scan: &Scan<&[u8], &[u8], impl Layout>,
) {
    let scores = || {
        scoring::compose(
            scoring::symbols::Equality::new(3i32, -2),
            scoring::gaps::Affine {
                open: -5,
                extend: -1,
            },
            scoring::equiv::Equality {},
        )
    };

    let mut group = criterion.benchmark_group(format!("pairwise/{region}/{layout}"));
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(scan.layout().len() as u64));

    for size in CHUNK_SIZES {
        let mut engine = Engine::new(Best::new(), TraceMatrix::new(), scores());
        engine.set_chunk_size(size);
        group.bench_function(BenchmarkId::new("Best", size.rows()), |bencher| {
            bencher.iter(|| engine.scan(black_box(scan)));
        });
    }

    for size in CHUNK_SIZES {
        let mut engine = Engine::new(AllOptimal::new(100), TraceMatrix::new(), scores());
        engine.set_chunk_size(size);
        group.bench_function(BenchmarkId::new("AllOptimal", size.rows()), |bencher| {
            bencher.iter(|| engine.scan(black_box(scan)));
        });
    }
    group.finish();
}

fn pairwise(criterion: &mut Criterion) {
    let fixture = include_bytes!("inverted_repeats.fasta");
    let mut records = Vec::new();
    fasta::Reader::new(fixture.as_slice())
        .expect("open benchmark FASTA")
        .read_to_end(&mut records)
        .expect("read benchmark FASTA");
    assert!(!records.is_empty(), "empty benchmark FASTA");

    for record in &records {
        let (forward, reverse) = strands(record);
        let (forward, reverse) = (forward.as_slice(), reverse.as_slice());

        let full = Scan::full(forward, reverse).expect("full benchmark layout");
        bench_scan(criterion, record.id(), "full", &full);

        let triangle =
            Scan::above_anti_diagonal(forward, reverse, 3).expect("anti-diagonal benchmark layout");
        bench_scan(criterion, record.id(), "triangle", &triangle);
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .without_plots()
        .warm_up_time(Duration::from_millis(400))
        .measurement_time(Duration::from_secs(2))
        .sample_size(10);
    targets = pairwise
}
criterion_main!(benches);
