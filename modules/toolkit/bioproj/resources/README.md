# bioproj test fixtures

These synthetic JSON documents are shared by the Rust and Python tests. They
use schema `0.0.1` and contain no actual Asset files.

`valid/` contains complete projects:

- [all_variants.json](valid/all_variants.json) covers every record variant.
- [empty_domains.json](valid/empty_domains.json) has empty provenance, data, and design domains.
- [equivalent_datasets.json](valid/equivalent_datasets.json) uses overlapping representations of one Acquisition.
- [shared_local_ids.json](valid/shared_local_ids.json) reuses local IDs across entity families.
- [sequencing_project.json](valid/sequencing_project.json) shows shared biological origins, distinct preparations, repeated measurements, and their FASTQ layouts.

Each `invalid/<name>.json` has one intended defect and a hand-written
`<name>.error.json` with the expected public diagnostic. The loader reports the first error. `unsupported_major_schema.json` also has an invalid domain to
check that schema dispatch comes first.
