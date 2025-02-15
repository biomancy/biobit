mod bundle;
pub mod interval_tree;

pub use bundle::IntoPyBundle;

use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_submodule(&interval_tree::construct(
            py,
            &format!("{name}.interval_tree"),
        )?)?
        .finish();

    Ok(module)
}
