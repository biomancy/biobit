use biobit_bioproj_rs as rs;
use pyo3::prelude::*;

use super::acquisition::{AcquisitionIdInput, AcquisitionInput};
use super::library::{LibraryIdInput, LibraryInput};
use super::sample::{PySample, PySampleId};
use super::source::{PySource, PySourceId};
use crate::utils::lookup::{define_family_record_index, define_homogeneous_record_index};

define_homogeneous_record_index!(
    PySourceIndex(rs::provenance::Provenance) as "SourceIndex",
    record: PySource,
    id: PySourceId as "SourceId",
    rust: rs::provenance::Source,
    lookup: source,
    iter: sources
);

define_homogeneous_record_index!(
    PySampleIndex(rs::provenance::Provenance) as "SampleIndex",
    record: PySample,
    id: PySampleId as "SampleId",
    rust: rs::provenance::Sample,
    lookup: sample,
    iter: samples
);

define_family_record_index!(
    PyLibraryIndex(rs::provenance::Provenance) as "LibraryIndex",
    record: LibraryInput,
    id: LibraryIdInput as "LibraryId",
    id_ref: rs::provenance::LibraryIdRef<'_>,
    rust: rs::provenance::Library,
    lookup: library,
    iter: libraries
);

define_family_record_index!(
    PyAcquisitionIndex(rs::provenance::Provenance) as "AcquisitionIndex",
    record: AcquisitionInput,
    id: AcquisitionIdInput as "AcquisitionId",
    id_ref: rs::provenance::AcquisitionIdRef<'_>,
    rust: rs::provenance::Acquisition,
    lookup: acquisition,
    iter: acquisitions
);
