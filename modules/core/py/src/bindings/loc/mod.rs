use pyo3::prelude::*;

pub use crate::loc::chain_interval::{IntoPyChainInterval, PyChainInterval};
use crate::utils::ImportablePyModuleBuilder;
pub use biobit_core_rs::loc::{ChainInterval, Contig, Interval, IntervalOp, Orientation, Strand};
pub use interval::{IntoPyInterval, PyInterval};
pub use orientation::{IntoPyOrientation, PyOrientation};
pub use per_orientation::PyPerOrientation;
pub use per_strand::PyPerStrand;
pub use strand::{IntoPyStrand, PyStrand};

mod chain_interval;
mod interval;
pub mod mapping;
mod orientation;
mod per_orientation;
mod per_strand;
mod strand;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyStrand>()?
        .add_class::<PyOrientation>()?
        .add_class::<PyPerOrientation>()?
        .add_class::<PyPerStrand>()?
        .add_class::<PyInterval>()?
        .add_class::<PyChainInterval>()?
        .add_submodule(&mapping::construct(py, &format!("{name}.mapping"))?)?
        .finish();

    Ok(module)
}
