# bioproj

`biobit-bioproj-rs` is the Rust core for immutable biological-project manifests.

The current implementation models released provenance through a logical,
demultiplexed run:

```text
Source <- Sample <- Library <- Assay <- Run
```

Samples may pool multiple sources and libraries may pool multiple samples. The
implemented concrete library types are `provenance::library::illumina::DnaLibrary` and
`provenance::library::illumina::CdnaLibrary`; the latter has an explicit
`provenance::library::strandedness::Strandedness`, including `Unknown`.
`provenance::assay::illumina::SingleEndSequencing` and `PairedEndSequencing` own the
compatibility contract and accept either concrete library type.
`provenance::run::illumina::SingleEndSequencing` and `PairedEndSequencing` then own the
matching typed assay parents.

Assets remain a separate domain and are children of runs. `Asset::Fastq` is a
single-file FASTQ format; `Asset::PairedFastq` preserves distinct read-one and
read-two file locations. Their typed input contracts currently accept matching
single- and paired-end Illumina runs without coupling the format name to
Illumina itself. `Assets::new(&provenance, assets)` validates the asset-to-run
relationship and ID uniqueness. `Project` owns the resulting asset collection
alongside its provenance and revalidates it as part of the released graph.

Design is a separate logical overlay. `DesignUnit` holds a non-empty set of
raw assay IDs, allowing it to pool heterogeneous assay types. `Design` has
`TwoUnits`, `TwoGroups`, `MatchedPairs`, and `UnitSet` variants.
`MatchedPairs` uses ordered two-element unit tuples, so `(A, B)` differs from
`(B, A)`. `Designs::new(&provenance, units, designs)` validates assay and
design-unit references; `Project` owns the resulting overlay.

`Provenance` validates raw-ID uniqueness, material references, and the typed
library-to-assay-to-run relationships. `Project` owns provenance, assets, and
design, validates their cross-domain ID uniqueness, and supports complete JSON
round-trips. Workspace manifests and storage adapters are intentionally outside
this slice.
