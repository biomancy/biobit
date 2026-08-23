//! Immutable biological-project manifests.
//!
//! This initial crate models released provenance through acquisitions:
//! [`provenance::Source`] <- [`provenance::Sample`] <-
//! [`provenance::Library`] <- [`provenance::Acquisition`]. [`data::Dataset`]
//! records bind complete file layouts to acquisitions, while [`data::Asset`]
//! records describe storage independently. [`Project`] owns the released
//! provenance, data, and [`Design`] domains.
//!
//! It currently implements P5/P7 libraries with a single tagged DNA- or
//! RNA-derived input, single- and paired-end Illumina acquisitions, single-file
//! FASTQ assets, FASTQ dataset layouts, initial experimental-design
//! topologies, and a TOML manifest with a closed JSON adapter. Detailed wet-lab
//! protocol provenance and physical acquisition batching remain outside this slice.

pub mod data;
pub mod design;
pub mod manifest;
mod primitives;
mod project;
pub mod provenance;
mod validation;

pub use design::{Design, DesignId, DesignUnit, DesignUnitId, Designs};
pub use manifest::{Adapter, Location, Manifest, ProjectHeader, Schema};
pub use primitives::{IsEmpty, Lookup, Meta, MetaVal, NonEmpty, UntypedId, Uri};
pub use project::{Project, ProjectId};
