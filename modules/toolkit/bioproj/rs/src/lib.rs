//! Immutable biological-project manifests.
//!
//! This initial crate models released provenance through logical runs:
//! [`provenance::Source`] <- [`provenance::Sample`] <-
//! [`provenance::Library`] <- [`provenance::Assay`] <-
//! [`provenance::Run`]. [`Asset`] records are separate children of
//! [`provenance::Run`], while [`Project`] owns a complete released graph
//! together with its [`Design`] overlay.
//!
//! It currently implements Illumina DNA and cDNA libraries plus single- and
//! paired-end Illumina sequencing assays and runs, `Fastq` and `PairedFastq`
//! assets, and initial experimental-design topologies. Workspace manifests,
//! storage adapters, and detailed wet-lab protocol provenance are deliberately
//! outside this slice.

pub mod asset;
pub mod design;
mod primitives;
mod project;
pub mod provenance;
mod validation;

pub use asset::{Asset, AssetId, Assets};
pub use design::{Design, DesignId, DesignUnit, DesignUnitId, Designs};
pub use primitives::{Id, Meta, MetaVal};
pub use project::{Project, ProjectId};
