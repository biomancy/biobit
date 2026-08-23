//! Typed lookup within a resolved provenance graph.

use super::acquisition::illumina::{
    PairedEndSequencing, PairedEndSequencingId, SingleEndSequencing, SingleEndSequencingId,
};
use super::library::p5p7;
use super::{
    Acquisition, AcquisitionId, AcquisitionIdRef, Library, LibraryId, LibraryIdRef, Provenance,
    Sample, SampleId, Source, SourceId,
};
use crate::primitives::{impl_checked_lookup, impl_direct_lookup, impl_variant_lookup};

impl_direct_lookup!(Provenance, SourceId, Source, sources);
impl_direct_lookup!(Provenance, SampleId, Sample, samples);
impl_checked_lookup!(Provenance, LibraryId, Library, libraries);
impl_checked_lookup!(Provenance, LibraryIdRef<'_>, Library, libraries);
impl_checked_lookup!(Provenance, AcquisitionId, Acquisition, acquisitions);
impl_checked_lookup!(Provenance, AcquisitionIdRef<'_>, Acquisition, acquisitions);

impl_variant_lookup!(
    Provenance,
    p5p7::LibraryId,
    p5p7::Library,
    libraries,
    Library::P5P7
);
impl_variant_lookup!(
    Provenance,
    SingleEndSequencingId,
    SingleEndSequencing,
    acquisitions,
    Acquisition::IlluminaSingleEndSequencing
);
impl_variant_lookup!(
    Provenance,
    PairedEndSequencingId,
    PairedEndSequencing,
    acquisitions,
    Acquisition::IlluminaPairedEndSequencing
);
