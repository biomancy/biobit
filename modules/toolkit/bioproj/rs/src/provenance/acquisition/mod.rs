//! Logical data acquisitions grouped by measurement family.

mod id;
pub mod illumina;
mod root;

pub use id::{AcquisitionId, AcquisitionIdRef};
pub use root::{Acquisition, AcquisitionKind};
