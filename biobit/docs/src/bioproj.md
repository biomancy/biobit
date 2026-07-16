# bioproj

## 1. Overview and Philosophy

`bioproj` is a strict, immutable data manifest for post-acquisition biological data and its accompanying scientific intent. It provides a generalized topological graph capable of securely modeling diverse measurement modalities while explicitly declaring the study's experimental design. Its overarching objective is to serve as the canonical, machine-readable standard for depositing biological data into public archives.

The architecture enforces a rigorous separation of concerns. Downstream workflow engines must treat `bioproj` as a read-only dependency. They must reuse its stable identifiers for cross-module data referencing and rely entirely on its declarative design to guide the topology of the analytical execution.

## 2. The Core Domains

The data model represents a topological flow from a biological source to measured assets, overlaid with scientific intent. To isolate varying lifecycles and ownership boundaries, the system is divided into three distinct bounded contexts.

### 2.1 Provenance

The `provenance` domain models the physical and technical provenance of the experiment. All relationships are defined by the child (downstream node) pointing to the parent (upstream node).

**The Biological DAG (Many-to-Many):**

* **Source:** The biological root (e.g., donor, cell line, pathogen). Encapsulates stable invariants such as genotype and strain.
* **Sample:** Physical material extracted from one or more `Sources`. It holds an array of source IDs, natively supporting the modeling of complex biological mixtures.
* **Library:** A molecular or analyte preparation derived from one or more `Samples`. It holds an array of sample IDs to explicitly model the physical pooling of material prior to assay generation.

**The Technical Tree (One-to-Many):**

* **Assay:** The measurement methodology applied to exactly one `Library`. It holds a scalar library ID and dictates the expected technical topology (e.g., dRNA-seq).
* **Run:** A logical record for demultiplexed data associated with exactly one `Assay`. It holds a scalar assay ID. It does not currently model a shared physical acquisition event such as a flowcell lane or MS injection.

**Polymorphic Tagged Variants:** `bioproj` enforces structural integrity at the level of `Assay` and `Run` via tagged variants (enums). An `Assay::ChIPSeq` strictly dictates a different downstream technical topology than an `Assay::MassSpec`. This guarantees that modality-specific constraints are mathematically enforced by the parser prior to materialization.

### 2.2 Assets

The `assets` domain decouples physical data from the biological topology. The biological graph remains unchanged when files are migrated across storage environments.

* **Asset:** Represents the physical file artifact. It holds a scalar run ID and environment-specific locators (e.g., URIs). Optional file-specific details, such as a checksum, may be recorded in its `meta` map when required.

### 2.3 Design

The `design` domain applies a logical overlay to define the scientific hypotheses. It defines topological contrasts strictly without dictating the software parameters or algorithms used to execute them.

* **DesignUnit:** A named, non-empty set of `Assays` logically pooled for computation.
* **Design:** A tagged union defining experimental topologies. Supported variants include:
  * `TwoUnits`: Direct case versus control contrast.
  * `TwoGroups`: Two-arm analyses without strict subject-level matching.
  * `MatchedPairs`: Paired mathematical correspondence.
  * `Set`: Unordered multivariate modeling.

### 2.4 The Metadata Escape Hatch

To accommodate opaque metadata all `bioproj` entities support an auxiliary `meta` key-value map for arbitrary strings.

## 3. The Workspace Paradigm (`bioproj.toml`)

Each `bioproj` workspace is fully resolved via a primary TOML manifest file. It acts as the entry point for the parser, declares the global schema version, and maps dependencies to their respective domain adapters.

This version does not bind domains through manifest-level hashes. When a file checksum is needed, it is recorded only on the corresponding `Asset` in its `meta` map. Workspace and domain hashing are reserved for a future revision.

```toml
[project]
id = "PROJ1"
schema_version = "0.0.1"
description = "A multi-modal bioproj workspace"
meta = { "Author" = "Vladimir Lenin & Adam Smith" }

[provenance]
location = { adapter = "json", uri = "file://provenance.json" }

[assets]
location = { adapter = "json", uri = "file://assets.json" }

[design]
location = { adapter = "json", uri = "file://design.json" }

```

## 4. Hard Graph Invariants & Validation

A `bioproj` workspace is structurally valid only if it successfully compiles into an in-memory graph satisfying the following criteria:

1. **Referential Integrity:** All relational IDs must resolve to an existing entity defined within the workspace. Dangling pointers are strictly prohibited.
2. **Strict Path Cardinality:** Every node in the physical graph must trace downstream to at least one physical `Asset`.
3. **Logical Isolation:** Within the expanded physical graph of a single `Design` contrast, an `Assay` cannot appear in opposing conceptual arms simultaneously.

### 4.1 Namespacing and Identifiers

All `bioproj` identifiers are bound by the following structural constraints:

* **Character Set:** All identifiers must strictly conform to the regex `[A-Za-z0-9_-]+`. This eliminates escaping collisions in downstream dataframe environments.
* **Uniqueness:** An identifier must be absolutely unique across *all* domains within a single workspace. For example, a `Source` and an `Assay` cannot share the ID `WT_01`. Project identifiers must be unique globally.

To construct globally unique keys for a project member, downstream systems must use a composite key `(project_id, local_id)`.

## 5. Serialization and Adapters

`bioproj` relies on the Ports and Adapters pattern to operate on payloads stored in diverse formats.

**JSON Adapter**: A normalized, flat JSON layout.

Example for `provenance.json`:

```json
{
  "sources": [
    {"id": "SRC1", "description": "Donor 1"}
  ],
  "samples": [
    {"id": "SMP1", "description": "Sample 1", "sources": ["SRC1"]}
  ],
  "libraries": [
    {"id": "LIB1", "samples": ["SMP1"]}
  ],
  "assays": [
    {"id": "ASY1", "library": "LIB1", "type": "RNASeq"}
  ],
  "runs": [
    {"id": "RUN1", "assay": "ASY1"}
  ]
}

```

Example for `assets.json`:

```json
{
  "assets": [
    {
      "id": "AST1",
      "run": "RUN1",
      "format": "Fastq",
      "uri": "s3://bucket/run1_R1.fq.gz",
      "meta": {"checksum": "sha256:9e107d9d372bb6826bd81d3542a419d6"}
    }
  ]
}

```

Example for `design.json`:

```json
{
  "units": [
    {
      "id": "UNIT_CTRL",
      "assays": ["ASY1"]
    },
    {
      "id": "UNIT_TREAT",
      "assays": ["ASY2"]
    }
  ],
  "designs": [
    {
      "id": "DES1",
      "type": "TwoUnits",
      "control": "UNIT_CTRL",
      "treatment": "UNIT_TREAT"
    }
  ]
}

```
