//! Logical data acquisitions grouped by measurement family.

pub mod illumina;
mod root;
mod validate;

pub use root::{Acquisition, AcquisitionId, AcquisitionIdRef, AcquisitionKind};
pub(crate) use validate::validate;
