use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

pub use pileup::{PyPileup, PySparsePileup};
pub use reat::PyReat;
pub use result::PySelectedPileup;
pub use task::PyTask;

mod pileup;
mod reat;
mod result;
pub mod selection;
mod task;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyReat>()?
        .add_class::<PyTask>()?
        .add_class::<PyPileup>()?
        .add_class::<PySparsePileup>()?
        .add_class::<PySelectedPileup>()?
        .add_submodule(&selection::construct(py, &format!("{name}.selection"))?)?
        .finish();

    Ok(module)
}
