# bioproj

## 1. Overview and scope

`bioproj` is an immutable, machine-readable description of a released
biological data project. It records the provenance graph, acquired data, and
scientific intent without prescribing an analysis workflow, software
parameters, or algorithms. Downstream systems consume it read-only and reuse
its stable identifiers when referring to project members.

The initial focus is a well-specified released description. A draft-authoring
lifecycle may later provide more permissive construction and incremental
validation, but it is not part of the first core API. A released graph can
already be assembled in code and validated. Persisting it is an explicit
manifest-and-adapter operation; the resolved `Project` is not itself a wire
format.

### Current Rust-core scope

The current Rust core implements the released graph through logical `Run` and
the initial asset formats:

```text
Source <- Sample <- Library <- Assay <- Run <- Asset
```

It currently supports concrete P5/P7-adapted DNA and RNA-derived cDNA fragment
libraries, Illumina single-end and paired-end sequencing assays and runs,
generic `Fastq` and `PairedFastq` asset formats, the initial design topologies,
and a complete released `Project` aggregate. It also implements a TOML manifest
and a closed JSON adapter that assemble those domains into a project. The rest
of this document distinguishes that implemented slice from the intended wider
architecture.

## 2. Core domains

The model follows physical and technical flow from biological material toward
measured data. Every edge is held by the child (downstream) node and points to
its parent (upstream) node.

### 2.1 Provenance

**Biological DAG (many-to-many):**

- **Source:** A biological root such as a donor, cell line, or pathogen. It
  has a required `organism` string. More detailed biology, such as genotype or
  strain, can be placed in `meta` when it is not needed to determine graph
  structure.
- **Sample:** Physical material extracted from one or more sources. Its
  non-empty `sources` set represents mixtures naturally.
- **Library:** A terminal prepared material derived from one or more samples.
  Its non-empty `samples` set represents physical pooling before acquisition.

Experimental treatments and conditions specific to collected material belong in a `Sample`’s `meta`, while stable source-wide history may be recorded on the corresponding `Source`.

Libraries are concrete tagged variants, not a generic core plus a bag of
structural attributes. Variants describe the prepared material and the stable
physical interface presented to an assay, rather than a vendor or preparation
kit. The implemented types live under `provenance::library::p5_p7`:

- `DnaLibrary` (serialized as `P5P7Dna`): a P5/P7-adapted fragment library
  prepared from DNA.
- `CdnaLibrary` (serialized as `P5P7Cdna`): a P5/P7-adapted fragment library
  prepared from RNA-derived cDNA.

Both carry non-empty `molecule` and `selection` sets. `P5P7Cdna` also has
an explicit `provenance::library::strandedness::Strandedness` value: `Unknown`,
`Unstranded`, `Forward`, or `Reverse`. `Unknown` is a real declared state,
rather than a missing optional field; it remains distinct from an intentionally
unstranded library.

P5/P7 is the acquisition compatibility boundary shared by many preparation
protocols. Indexing is not part of the type: indexed and unindexed libraries
have the same terminal interface, and the current graph models demultiplexed
logical outputs rather than the physical multiplexing operation.

New library variants should be introduced only with a real material schema and
defined assay compatibility. There are deliberately no empty placeholders for
Nanopore, PacBio, mass spectrometry, or other future modalities.

**Assay compatibility (one library to many assays):**

An assay describes an acquisition method applied to exactly one library. Its
input contract owns compatibility; a library does not advertise an open-ended
set of capabilities. This keeps a concrete library concerned only with its
prepared material and lets a new assay declare the library types it accepts.

The implemented Illumina assays live under `provenance::assay::illumina`:

- `SingleEndSequencing`
- `PairedEndSequencing`

Both accept the closed input union `SequencingInput`, which is either a
`provenance::library::p5_p7::DnaLibraryId` or a
`provenance::library::p5_p7::CdnaLibraryId`. Thus either library can be
sequenced with either read layout, but a mass-spectrometry library would not
become acceptable merely by sharing a raw string ID.

Rust uses distinct ID types for each concrete library and assay type. On the
wire, an assay stores only its raw `library` ID; parsing resolves that ID to the
actual tagged library and constructs the corresponding typed input. The edge
therefore does not duplicate a library type tag that could disagree with the
library record.

The root enums list concrete variants directly, for example
`provenance::Library::P5P7Dna` and
`provenance::Assay::IlluminaSingleEndSequencing`.
The concrete structs and their IDs remain namespaced, without introducing a
provenance accessor for every combination.

**Logical technical outputs:**

- **Run:** A logical record for demultiplexed data associated with exactly one
  assay. Runs mirror the currently implemented Illumina assay layouts:
  `Run::IlluminaSingleEndSequencing` and
  `Run::IlluminaPairedEndSequencing`. Each concrete run holds the matching
  typed assay ID, so a paired run cannot be attached to a single-end assay.
  A run intentionally does not model a shared physical acquisition event such
  as a flowcell lane or an MS injection.

The library is a prepared-material checkpoint, not a general wet-lab protocol
language. If two materials share upstream processing and later diverge, model
them as two libraries with their common sample parent. A separate process
provenance language should only be added if a concrete use case cannot be
expressed this way.

### 2.2 Assets

The `assets` domain decouples data artifacts from biological topology. Every
asset is a child of a run: it owns a `run` reference rather than the run owning
a mutable list of assets. This permits appending assets in a later released
description without changing the parent provenance records.

The root `Asset` enum currently has two reusable file-format variants:

- `Asset::Fastq`: a single-file FASTQ asset. Its input contract currently
  accepts a single-end Illumina run, and can later be extended with other
  single-file FASTQ-producing runs such as direct-RNA or long-read sequencing.
- `Asset::PairedFastq`: a paired FASTQ asset. It has separate, non-empty
  `read1` and `read2` URI locator sets and currently accepts a paired-end
  Illumina run.

The shared `Uri` primitive currently accepts only `file:<filename>` locators.
Each filename identifies a file directly beside the containing `bioproj.toml`;
remote schemes, absolute paths, and nested paths are deferred until URI support
is backed by a dedicated crate. URI locators in one set therefore identify
equivalent local copies of the same file and are not a way to bundle unrelated
files. The read-one and read-two locator sets of a `PairedFastq` asset must not
overlap.

There is intentionally no generic or opaque asset fallback in this revision.
A new format should be added as a tagged variant once it has a real schema and
run compatibility contract. Checksums, when needed, belong in asset `meta`,
not in a manifest hash or dedicated checksum field.

`Assets` remains independently usable and does not own provenance.
Construction, and deserialization through
`Assets::deserialize_with_provenance`, take `&Provenance` so raw `run` IDs can
resolve to the corresponding typed run inputs. `Project` owns both domains in
a complete released description and revalidates their relationship.

### 2.3 Design

The `design` domain overlays scientific intent on the provenance graph. It
describes topological contrasts without dictating computational methods or
parameters. `Designs` does not own provenance: construction and deserialization
take `&Provenance` so every assay named by a design unit can be validated.
`Project` owns both domains in a complete released description.

- **DesignUnit:** A named, non-empty set of assays logically pooled for a
  computation. Its assay references use common raw IDs because a unit may pool
  assays of any concrete type.
- **TwoUnits:** A direct contrast between distinct `control` and `treatment`
  units.
- **TwoGroups:** A two-arm analysis without mandatory subject-level matching.
  Both `control` and `treatment` are non-empty, duplicate-free, and disjoint
  sets of units.
- **MatchedPairs:** Explicit one-to-one pairwise correspondence. Its JSON
  representation is ordered pairs of unit IDs, for example
  `[["UNIT_A", "UNIT_B"]]`, not `control`/`treatment` fields on every pair.
  Tuple order is preserved: `["UNIT_A", "UNIT_B"]` differs from
  `["UNIT_B", "UNIT_A"]`.
- **UnitSet:** A non-empty unordered collection of units for a joint analysis.

Blocked and more complex designs are intentionally out of the initial scope.

### 2.4 Auxiliary metadata

Every entity may carry a `meta` map with string or boolean values. `meta` is
for auxiliary annotations, not for describing the DAG: it must not determine
an entity's type, identity, parent references, compatibility, or validation.
Those properties belong in explicit fields and tagged variants. This is why
the name remains `meta` rather than `attrs`.

## 3. Project manifest

`Manifest::load(path)` reads the primary `bioproj.toml` manifest and resolves
its three domain payloads into a `Project`. The manifest is strictly an
orchestration record: it selects a schema and closed adapter for each payload;
it does not add graph validation beyond parsing and file I/O. The domain types
perform their normal released-graph validation while loading.

The `[project]` `schema` field parses into a `Schema { major, minor, patch }`.
The initial loader supports exactly `0.0.1` and rejects another schema before
reading any domain payload. All of `[provenance]`, `[assets]`, and `[design]`
are required, including when a payload is explicitly empty.

`Adapter` is a closed enum and currently has one variant, `json`. Every
location currently uses `file:<filename>` and resolves that filename relative
to the directory containing `bioproj.toml`. `file://…`, absolute paths,
subdirectories, remote schemes, and adapter-specific configuration are not
accepted in this initial release.

There are no manifest-level content hashes. If a checksum is required, store it
only on the corresponding asset in `meta`; domain and project hashing can be
introduced in a future revision.

```toml
[project]
id = "PROJ1"
schema = "0.0.1"
description = "A multi-modal bioproj project"
meta = { "Author" = "Vladimir Lenin & Adam Smith" }

[provenance]
location = { adapter = "json", uri = "file:provenance.json" }

[assets]
location = { adapter = "json", uri = "file:assets.json" }

[design]
location = { adapter = "json", uri = "file:design.json" }
```

## 4. Released-graph validation

The current core validates a released provenance graph when its domains are
constructed or loaded, validates `Assets` and `Designs` when they are resolved
against provenance, and validates their complete combination when constructing
`Project`. The manifest adds no extra graph invariants:

1. **Referential integrity:** A sample's source IDs, a library's sample IDs,
   a run's assay ID, a design unit's assay IDs, and a design's unit IDs must
   resolve.
2. **Typed compatibility:** An assay's library input, a run's assay parent,
   and an asset's run parent must resolve to their matching concrete variants.
3. **Required collections:** Parent ID sets, asset URI locator sets, and the
   current P5/P7 `molecule` and `selection` sets are non-empty and
   duplicate-free. Design units, groups, matched-pair collections, and unit
   sets are subject to the same structural rule; two groups must be disjoint
   and a unit may occur in at most one matched pair.
4. **Identifier uniqueness:** Raw IDs are unique across sources, samples,
   libraries, assays, and runs in a provenance graph. `Assets::new` additionally
   rejects an asset ID that conflicts with either provenance or another asset;
   `Designs::new` does the same for design-unit and design IDs. `Project`
   completes this validation across its project ID, provenance, assets, design
   units, and designs.

The current core does not yet require every run to have an asset, or validate
design-arm isolation. Those are candidate release-time checks once their
semantics are specified; they are not a precondition for a separate draft model
today.

### 4.1 Identifiers

All identifiers conform to `[A-Za-z0-9_-]+`. Rust uses distinct ID wrapper
types where the expected concrete entity type matters, while the serialized
format uses their shared string representation. `Project` has its own typed
project ID; downstream systems can use `(project_id, local_id)` as a globally
scoped key.

## 5. Domain serialization and adapters

The current JSON representation is normalized and flat. `Provenance`, `Assets`,
and `Designs` serialization are deterministic because their entity collections
are ordered by ID. Parent fields are raw IDs on the wire; they are resolved
against the parent domain during deserialization. `Project` is deliberately not
serializable or deserializable: it is the resolved aggregate produced by
`Manifest::load` or `Project::new`, not a competing monolithic wire format.
Writing a code-built project will later be an explicit manifest-and-adapter
operation that receives the output locations.

```json
{
  "sources": [
    {
      "id": "SRC1",
      "organism": "Homo sapiens",
      "description": "Donor 1"
    }
  ],
  "samples": [
    {
      "id": "SMP1",
      "sources": ["SRC1"],
      "description": "Sample 1"
    }
  ],
  "libraries": [
    {
      "type": "P5P7Dna",
      "id": "LIB_DNA",
      "samples": ["SMP1"],
      "molecule": ["DNA"],
      "selection": ["none"]
    },
    {
      "type": "P5P7Cdna",
      "id": "LIB_CDNA",
      "samples": ["SMP1"],
      "molecule": ["cDNA"],
      "selection": ["poly-A"],
      "strandedness": "Forward"
    }
  ],
  "assays": [
    {
      "type": "IlluminaSingleEndSequencing",
      "id": "ASY_DNA",
      "library": "LIB_DNA"
    },
    {
      "type": "IlluminaPairedEndSequencing",
      "id": "ASY_CDNA",
      "library": "LIB_CDNA"
    }
  ],
  "runs": [
    {
      "type": "IlluminaSingleEndSequencing",
      "id": "RUN_DNA",
      "assay": "ASY_DNA"
    },
    {
      "type": "IlluminaPairedEndSequencing",
      "id": "RUN_CDNA",
      "assay": "ASY_CDNA"
    }
  ]
}
```

An assets payload is serialized separately and resolved with its provenance:

```json
{
  "assets": [
    {
      "type": "Fastq",
      "id": "AST_DNA",
      "run": "RUN_DNA",
      "locations": ["file:run_dna.fq.gz"],
      "meta": {"checksum": "sha256:..."}
    },
    {
      "type": "PairedFastq",
      "id": "AST_CDNA",
      "run": "RUN_CDNA",
      "read1": ["file:run_cdna_R1.fq.gz"],
      "read2": ["file:run_cdna_R2.fq.gz"]
    }
  ]
}
```

A design payload is likewise serialized separately and resolved with its
provenance:

```json
{
  "units": [
    {
      "id": "UNIT_CTRL",
      "assays": ["ASY_DNA"]
    },
    {
      "id": "UNIT_TREAT",
      "assays": ["ASY_CDNA"]
    }
  ],
  "designs": [
    {
      "type": "TwoUnits",
      "id": "DES_CONTRAST",
      "control": "UNIT_CTRL",
      "treatment": "UNIT_TREAT"
    },
    {
      "type": "MatchedPairs",
      "id": "DES_PAIRED",
      "pairs": [["UNIT_TREAT", "UNIT_CTRL"]]
    },
    {
      "type": "UnitSet",
      "id": "DES_JOINT",
      "units": ["UNIT_CTRL", "UNIT_TREAT"]
    }
  ]
}
```

The JSON adapter is the only implemented adapter. It loads provenance first,
then assets and design with that resolved provenance, and finally constructs
the project from the manifest header. New adapters are explicit future
`Adapter` variants rather than externally registered implementations.
