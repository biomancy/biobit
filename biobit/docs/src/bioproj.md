# bioproj

`bioproj` is an immutable, machine-readable description of a released
biological data project. It records biological provenance, acquired data, and
scientific intent without prescribing analysis software or parameters.
Downstream systems consume the project read-only and reuse its identifiers.

The first API focuses on released descriptions. Draft authoring may later have
a more permissive lifecycle, but it is deliberately separate from the resolved
`Project` described here.

## Model

The material and measurement path is:

```text
Source <- Sample <- Library <- Acquisition
```

Each downstream object stores the IDs of its parents. Assets are independent
storage records; a Dataset connects a complete file layout to one Acquisition.

| Level | Meaning | Typical information |
| --- | --- | --- |
| **Source** | A stable biological entity represented across one or more samples. | Organism, genotype, strain, cell type, tissue of origin, or source-wide history. |
| **Sample** | Material collected or manipulated from one or more sources under one experimental condition. | Treatment, time point, viral MOI, sampled tissue, or biological replicate. |
| **Library** | Prepared material exposing a technical interface to compatible acquisitions. | Input molecule, extraction, enrichment, selection or depletion, strandedness, kit, or UMIs. |
| **Acquisition** | One measurement of one or more compatible libraries. | Platform, read layout, instrument, date, or acquisition labels. |
| **Asset** | One immutable stored file. | Location, checksum, compression, or format details. |
| **Dataset** | A sealed, analysis-ready arrangement of assets for one acquisition. | Representation, quality selection, or other dataset-level context. |

The last column shows conceptual ownership. Information with a stable model
belongs in explicit fields; less structured annotations can remain in `meta`.

### Provenance

A Source identifies one biological entity with a stable biological state. If
material contains several organisms or otherwise mixes distinct entities,
those entities remain separate Sources and the Sample references all of them.
A Sample can therefore have several Source parents, and a Library can pool
material from several Samples.

A Library is named by the technical interface it presents downstream, not by a
vendor kit or its starting molecule. The current `Library::P5P7` variant has
one tagged input, `FromDna` or `FromRna`; RNA input also records
`Strandedness`. This input compresses the relevant preparation history rather
than trying to model every wet-lab reaction. Differently prepared material is
represented as separate Libraries.

An Acquisition declares the Library types it accepts and references a
non-empty set of Libraries. This permits intentional pooling of compatible
Libraries prepared with different kits or at different times. Repeating the
measurement creates another Acquisition.

For now, Acquisition combines a reusable assay specification with one concrete
execution. Those concepts may later become separate nodes if repeated methods
need to be described once and reused. Physical multiplexing and demultiplexing
are outside the current graph.

The implemented acquisition variants are Illumina single-end and paired-end
sequencing. Both accept P5/P7 Libraries regardless of whether their input was
DNA or RNA. Rust preserves this compatibility with concrete ID types; the JSON
form stores only plain string IDs and resolves them while loading.

### Data

The `data` domain owns both Assets and Datasets. It is validated against
Provenance when constructed or loaded, then can be used independently through
its shallow Acquisition references. Rust exposes the record types under
`data::asset` and `data::dataset`.

An Asset describes storage only. It has no Acquisition reference and makes no
claim about how its contents should be interpreted. The current
`Asset::Fastq` variant represents exactly one FASTQ file at one URI location.

A Dataset supplies the missing scientific boundary. It references exactly one
compatible Acquisition and asserts that its listed Assets are the complete
contents of that representation:

- `Dataset::Fastq` contains a non-empty set of independent FASTQ Assets.
- `Dataset::PairedFastq` contains a non-empty set of ordered `(read1, read2)`
  FASTQ Asset pairs. Each Asset occurs at most once in that Dataset.

Pairing therefore belongs to the Dataset, not to a special paired-file Asset.
The typed Dataset inputs currently connect `Fastq` to single-end Illumina
acquisitions and `PairedFastq` to paired-end Illumina acquisitions. Other
FASTQ-producing technologies can extend these Dataset-owned compatibility
contracts later.

One Acquisition may have several Datasets when the same data is stored or
arranged differently. Datasets may overlap, for example when a quality-curated
view excludes a failed lane, but such subsets should be explicit in the
Dataset description or metadata rather than treated as the default. An Asset
may be reused only by Datasets for the same Acquisition.

`Uri` currently accepts only `file:<filename>`, referring to a file directly
beside `bioproj.toml`. Remote schemes, absolute paths, and nested paths are not
yet supported. Checksums belong in Asset `meta`; the manifest has no content
hashes.

### Design

Design describes scientific intent without choosing an analysis algorithm.
A `DesignUnit` is a named, non-empty set of Acquisitions logically pooled for
computation. Designs then arrange those units as:

- `TwoUnits`: one control and one treatment unit.
- `TwoGroups`: two non-empty, disjoint groups of units.
- `MatchedPairs`: ordered two-tuples of units. `(A, B)` and `(B, A)` differ,
  and a unit can occur in at most one pair.
- `UnitSet`: a non-empty unordered collection for joint analysis.

Blocked and more complex designs are outside the initial scope. Downstream
code selects a compatible Dataset for each Acquisition named by the design.

### Metadata

Every entity may carry a `meta` map with non-empty string keys and string or
boolean values. Metadata is auxiliary: identity, graph edges, compatibility,
and other validated semantics belong in explicit fields and tagged variants.

### Identifiers and lookup

Closed entity families such as Library, Acquisition, Asset, and Dataset use
the same identity model. An `UntypedId` identifies a workspace member, while a
family `Kind` identifies a concrete variant. A concrete ID carries both. Each
family also provides an owned `Id` union for stored heterogeneous references
and a borrowed `IdRef` union for inspecting or resolving them without cloning.
The corresponding root enum holds the full heterogeneous records.

Lookup preserves the precision of its key: an untyped ID returns the root
enum, while a concrete ID returns its concrete record. Typed lookup separates
an absent ID from an ID found under the wrong Kind. On the wire, references
remain plain ID strings; the owning domain restores their variants while
loading, avoiding a second type tag that could disagree with the target.

## Manifest

`Manifest::load(path)` reads `bioproj.toml`, loads three required domain
payloads, and constructs a resolved `Project`. The manifest only orchestrates
storage; domain and project types perform graph validation.

The schema is parsed into `Schema { major, minor, patch }`. The current loader
supports exactly `0.0.1`. Adapters form a closed enum with one implemented
variant, `json`.

```toml
[project]
id = "PROJ1"
schema = "0.0.1"
description = "Example project"
meta = { author = "Ada Example" }

[provenance]
location = { adapter = "json", uri = "file:provenance.json" }

[data]
location = { adapter = "json", uri = "file:data.json" }

[design]
location = { adapter = "json", uri = "file:design.json" }
```

`Project` is not itself serializable. It is the resolved aggregate produced by
the manifest or by code; writing one will remain an explicit
manifest-and-adapter operation.

## Released-project invariants

A released Project enforces:

1. Every Source, Sample, Library, Acquisition, Asset, Dataset, DesignUnit, and
   Design ID is unique across the project. IDs match `[A-Za-z0-9_-]+`.
2. Every graph reference resolves: Samples to Sources, Libraries to Samples,
   Acquisitions to Libraries, Datasets to Acquisitions and Assets, and Designs
   to their units.
3. Typed compatibility holds at Library-to-Acquisition and
   Acquisition-to-Dataset boundaries.
4. Required parent, asset, pair, group, and unit collections are non-empty and
   duplicate-free, with the topology-specific constraints described above.
5. Every Acquisition has at least one Dataset, every Asset belongs to at least
   one Dataset, and an Asset is never assigned across Acquisitions.

Design-arm isolation and a separate draft model remain future work.

## JSON payloads

The JSON adapter uses deterministic, flat domain payloads. Parent and member
references are plain IDs on the wire and regain their concrete types while the
domains are resolved.

`provenance.json`:

```json
{
  "sources": [
    {"id": "SRC1", "organism": "Homo sapiens"}
  ],
  "samples": [
    {"id": "SMP1", "sources": ["SRC1"]}
  ],
  "libraries": [
    {
      "type": "P5P7",
      "id": "LIB1",
      "samples": ["SMP1"],
      "input": {"type": "FromRna", "strandedness": "Forward"},
      "meta": {"selection": "poly-A"}
    }
  ],
  "acquisitions": [
    {
      "type": "IlluminaPairedEndSequencing",
      "id": "ACQ1",
      "libraries": ["LIB1"]
    }
  ]
}
```

`data.json`:

```json
{
  "assets": [
    {
      "type": "Fastq",
      "id": "AST_R1",
      "location": "file:reads_R1.fq.gz",
      "meta": {"checksum": "sha256:..."}
    },
    {
      "type": "Fastq",
      "id": "AST_R2",
      "location": "file:reads_R2.fq.gz"
    }
  ],
  "datasets": [
    {
      "type": "PairedFastq",
      "id": "DATA1",
      "acquisition": "ACQ1",
      "pairs": [["AST_R1", "AST_R2"]]
    }
  ]
}
```

`design.json`:

```json
{
  "units": [
    {"id": "UNIT_CTRL", "acquisitions": ["ACQ1"]}
  ],
  "designs": [
    {"type": "UnitSet", "id": "DES1", "units": ["UNIT_CTRL"]}
  ]
}
```
