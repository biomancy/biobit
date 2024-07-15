use derive_more::From;
use pyo3::prelude::*;

use biobit_core_rs::ngs::Layout;

use super::mates_orientation::PyMatesOrientation;
use super::strandedness::PyStrandedness;

#[pyclass(eq, ord, hash, frozen, name = "Layout")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From)]
pub enum PyLayout {
    /// Single-end sequencing library
    Single { strandedness: PyStrandedness },
    /// Paired-end library
    Paired {
        strandedness: PyStrandedness,
        orientation: PyMatesOrientation,
    },
}

impl From<PyLayout> for Layout {
    fn from(value: PyLayout) -> Self {
        match value {
            PyLayout::Single { strandedness } => Layout::Single {
                strandedness: strandedness.into(),
            },
            PyLayout::Paired {
                strandedness,
                orientation,
            } => Layout::Paired {
                strandedness: strandedness.into(),
                orientation: orientation.into(),
            },
        }
    }
}
