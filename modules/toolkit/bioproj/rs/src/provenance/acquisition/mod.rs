//! Logical data acquisitions grouped by measurement family.

pub mod illumina;
mod root;

pub use root::{Acquisition, AcquisitionId, AcquisitionIdRef, AcquisitionKind};
