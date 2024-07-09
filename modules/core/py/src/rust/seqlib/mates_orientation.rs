use pyo3::pyclass;

use biobit_core_rs::seqlib::MatesOrientation as RsMatesOrientation;

#[pyclass(eq, ord, hash, frozen)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(i8)]
pub enum MatesOrientation {
    Inward = 1,
    Outward = -1,
    Matching = 0,
}

impl From<MatesOrientation> for RsMatesOrientation {
    fn from(value: MatesOrientation) -> Self {
        match value {
            MatesOrientation::Inward => RsMatesOrientation::Inward,
            MatesOrientation::Outward => RsMatesOrientation::Outward,
            MatesOrientation::Matching => RsMatesOrientation::Matching,
        }
    }
}
