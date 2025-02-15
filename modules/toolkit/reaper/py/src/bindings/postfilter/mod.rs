use biobit_core_py::utils::ImportablePyModuleBuilder;
pub use nms::PyNMS;
use pyo3::prelude::*;

mod nms;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyNMS>()?
        .finish();

    Ok(module)
}
