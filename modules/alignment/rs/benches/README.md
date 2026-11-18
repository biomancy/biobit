# Pairwise alignment benchmarks

Each benchmark aligns a sequence to its own reverse complement. The suite runs
`Best` and `AllOptimal` on the full matrix and an anti-diagonal triangle, with
chunk sizes of 1, 16, 64, 128, 256, 512, and 1,024 rows.

## Run the benchmarks

Run the Criterion suite from the repository root.

```sh
pixi run -e rust cargo bench -p biobit-alignment-rs --bench pairwise
```

To run a subset, pass a regex that matches the case names. For example,
`pairwise/Nabp1/full/Best/256` is the full-matrix `Best` run for Nabp1 with
256 rows per chunk.

```sh
pixi run -e rust cargo bench -p biobit-alignment-rs --bench pairwise -- '/Best/'
pixi run -e rust cargo bench -p biobit-alignment-rs --bench pairwise -- '/AllOptimal/'
pixi run -e rust cargo bench -p biobit-alignment-rs --bench pairwise -- '/256$'
```

Add `-- --test` to the full-suite command for a quick optimized smoke check.
Debug runs of the benchmark target use 65-base prefixes.

## What we measure

Criterion reports time per scan and cells per second. Each timed call to
`Engine::scan` includes matrix initialization, scoring, endpoint tracking,
traceback, and alignment construction. The engine reuses its buffers between
scans. The FASTA is read and the reverse complement is prepared before timing.
`AllOptimal` uses a minimum score of 100.

After warm-up, Criterion aims to collect 10 samples per case in about two
seconds. Each sample runs the same number of scans, with at least one scan per
sample.

## The test sequences

We sampled inverted LINE-1 subsequences from DNA around the Nabp1 and Macir loci
and joined them into one 4,097-base record per locus.
[Benchmark FASTA](inverted_repeats.fasta) · [Original FASTA](payload.fasta)

The table uses zero-based, end-exclusive coordinates in the original FASTA.

| Source | Left arm | Right arm |
| --- | --- | --- |
| Nabp1 | `7805..8488` | `15118..15801` |
| Nabp1 | `8558..9241` | `18654..19337` |
| Nabp1 | `11431..12114` | `20302..20984` |
| Macir | `11581..12264` | `28691..29374` |
| Macir | `12468..13151` | `45957..46640` |
| Macir | `24854..25537` | `48859..49541` |
