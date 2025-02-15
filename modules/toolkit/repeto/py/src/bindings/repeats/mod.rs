use biobit_core_py::utils::ImportablePyModuleBuilder;
pub use inv::{PyInvRepeat, PyInvSegment};
use pyo3::prelude::PyModule;
use pyo3::{Bound, PyResult, Python};

mod inv;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyInvRepeat>()?
        .add_class::<PyInvSegment>()?
        .finish();

    Ok(module)
}
