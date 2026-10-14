//! Immutable released biological-project descriptions.
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
//! topologies, and a single-document JSON reader and writer. Acquisitions are
//! logical measurement outcomes; physical batching remains outside this graph.

pub mod data;
pub mod design;
mod error;
pub mod io;
pub mod lookup;
mod primitives;
mod project;
pub mod provenance;

pub use design::{Design, DesignId, DesignIdRef, DesignKind, Designs};
pub use error::{Error, Result};
pub use lookup::Lookup;
pub use primitives::{
    FilePath, IsEmpty, Layout, MatesOrientation, Meta, MetaVal, NonEmpty, OrderedSet, Root,
    Strandedness, Text, UntypedId,
};
pub use project::{Project, ProjectId};
