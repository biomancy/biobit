use pyo3::pyclass;

pub use biobit_core_rs::seqlib::strandedness::deduce;
use biobit_core_rs::seqlib::Strandedness as RsStrandedness;

#[pyclass(eq, ord, hash, frozen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i8)]
pub enum Strandedness {
    Forward = 1,
    Reverse = -1,
    Unstranded = 0,
}

impl From<Strandedness> for RsStrandedness {
    fn from(value: Strandedness) -> Self {
        match value {
            Strandedness::Forward => RsStrandedness::Forward,
            Strandedness::Reverse => RsStrandedness::Reverse,
            Strandedness::Unstranded => RsStrandedness::Unstranded,
        }
    }
}
