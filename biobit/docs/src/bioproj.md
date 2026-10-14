# bioproj

`bioproj` describes released biological data and the scientific context needed to use it.

## Project

A **Project** is an immutable description of a released collection of biological
data. It connects measurements and files to their biological context and intended
analyses through three parts:

- **Provenance** records biological origins, sample preparation, and measurements.
- **Data** connects measurements to complete collections of files.
- **Design** groups measurements for analysis and comparison.

Analysis applications reuse the Project's identifiers and Designs while managing
their own software configuration and results.

## Provenance

Biological material passes through four levels:

```text
Source -> Sample -> Library -> Acquisition
```

| Entity | Meaning |
| --- | --- |
| **Source** | A biological origin, such as a donor, cell line, or pathogen. |
| **Sample** | Material collected or manipulated under one experimental condition. |
| **Library** | Prepared material with a defined measurement interface. |
| **Acquisition** | One logical measurement of compatible Libraries. |

Each record references its parents by ID. A Sample can have several Sources,
a Library can pool several Samples, and an Acquisition can measure several
intentionally pooled Libraries.

For example, control and treated material from the same donor share a Source
but use different Samples. Different preparations produce separate Libraries.
Measuring a Library twice produces two Acquisitions.

Each Acquisition accepts only compatible Library types, as enforced by the Rust
type system. The model currently supports P5/P7 Illumina-like DNA or cDNA Libraries
and single-end or paired-end Illumina sequencing Acquisitions.

## Data

An **Asset** describes one immutable file and its location. A **Dataset** arranges
Assets into a complete representation of one compatible Acquisition and defines
the relationships between its files.

Rust types enforce compatibility between each Dataset layout and its Acquisition.

Currently, every Asset is one FASTQ file. The two Dataset layouts are:

| Dataset | Contents | Compatible Acquisition |
| --- | --- | --- |
| `Fastq` | One or more FASTQ files. | Illumina single-end sequencing. |
| `PairedFastq` | One or more ordered pairs of FASTQ files: read 1, then read 2. | Illumina paired-end sequencing. |

Each Asset appears at most once within a Dataset. Pairing belongs to the Dataset.

All Datasets for an Acquisition represent the same complete released
observations with the same interpretation. They may differ in file splitting or
encoding.

Every Acquisition has at least one Dataset, and every Asset belongs to at least
one Dataset. An Asset can be shared only by Datasets of the same Acquisition.

## Design

A **Unit** is a group of one or more Acquisitions pooled for computation.
For example, repeated sequencing measurements can form one Unit. Users define
Units for their analysis; `bioproj` makes no assumptions about biological
independence or replicate structure.

A **Design** arranges Units to express scientific intent:

| Design | Meaning |
| --- | --- |
| `UnitContrast` | One control Unit and one treatment Unit. |
| `GroupContrast` | Non-empty, disjoint control and treatment groups of Units. |
| `MatchedPairs` | One or more ordered Unit pairs; each Unit occurs in at most one pair. |
| `UnitSet` | A non-empty, unordered collection of Units for joint analysis. |

Units in a contrast or matched-pair Design must not share Acquisitions. They may
share biological ancestors.

`UnitSet` permits overlapping Units. Units and Acquisitions can be reused across
Designs.

A Design connects scientific intent to computational tools. For example, the same
`GroupContrast` can be used to study differential expression and differential
A-to-I RNA editing without redefining the input of each tool.

## Identity and annotations

A Project ID identifies one immutable released description. Changing that
description requires a new Project ID.

Each record is identified by its Project ID, entity family (such as Sample or
Acquisition), and local ID. For example, an Acquisition with Project ID `prj-1`
and local ID `acq-1` has identity `(prj-1, acquisition, acq-1)`. This identity 
should be globally unique.

For example, since variants in a family share one ID namespace, single-end and paired-end
Acquisitions cannot reuse an ID within a Project.

Every Design requires a name for downstream display and annotation. Names may
repeat. Supplied text fields must be non-empty printable ASCII with no leading
or trailing whitespace.

Records can carry a description and a `meta` map for additional context, such as
a Sample's treatment or a Library's preparation kit.

## Validation

Constructing or loading a Project checks references, compatibility, and collection
rules. Each domain checks its internal relationships; Project construction also
checks links from Data and Design to Provenance.

Required collections are non-empty and duplicate-free. Authors remain
responsible for biological meaning and Dataset equivalence.

## Input and output

JSON is the only supported storage format. One document holds the entire Project
description. Loading validates the description without opening or verifying
Asset files.

Asset locations can be absolute or relative file paths. When loading, relative
paths use the canonical document directory as their root. Roots must be existing
directories; Asset files need not exist. Assets expose absolute paths on demand.

Saving to another directory rewrites relative paths to keep them pointing to the
same files, including when the destination directory is a symlink. Saving fails
if a relative path cannot be formed. It does not move Asset files.
