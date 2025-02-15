use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

pub mod bam;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_submodule(&bam::construct(py, &format!("{name}.bam"))?)?
        .finish();

    Ok(module)
}
