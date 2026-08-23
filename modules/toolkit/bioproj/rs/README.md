# bioproj

`biobit-bioproj-rs` is the Rust core for immutable released biological-project
descriptions.

```text
Source <- Sample <- Library <- Acquisition
```

Libraries expose a technical interface to compatible Acquisitions. The current
P5/P7 Library records one DNA- or RNA-derived input path, and Illumina
single-end and paired-end Acquisitions can intentionally pool one or more P5/P7
Libraries. Acquisition currently combines the measurement specification and
one execution; those concepts may be separated later.

The `data` domain owns Assets and Datasets. Assets are independent storage
records: `Asset::Fastq` represents one file at one location and has no
provenance reference. A Dataset connects a complete stored representation to
exactly one compatible Acquisition. `Dataset::Fastq` groups independent FASTQ
files, while `Dataset::PairedFastq` stores ordered FASTQ ID pairs. The same
Acquisition may have several overlapping Datasets, but an Asset cannot be
reused across different Acquisitions.

Design is a logical overlay. Each `DesignUnit` pools one or more Acquisitions;
`TwoUnits`, `TwoGroups`, `MatchedPairs`, and `UnitSet` arrange those units into
analysis topologies.

`Project` owns the resolved provenance, data, and design domains. It has no
monolithic wire format. `Manifest::load("bioproj.toml")` loads the three
explicit payloads through the closed JSON adapter and constructs the validated
Project.
