use std::path::PathBuf;

use crate::project::PyProject;
use crate::utils::error::PyBioprojResult;
use biobit_bioproj_rs as rs;
use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
use pyo3::types::PyString;
use std::borrow::Borrow;

#[pyfunction]
fn read(path: PathBuf) -> PyBioprojResult<PyProject> {
    Ok(rs::io::json::read(path)?.into())
}

#[pyfunction]
#[pyo3(signature = (path, project, pretty = true))]
fn write(path: PathBuf, project: PyRef<'_, PyProject>, pretty: bool) -> PyBioprojResult<()> {
    rs::io::json::write(path, Borrow::<rs::Project>::borrow(&*project), pretty)?;
    Ok(())
}

fn json<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .finish();
    module.add_function(wrap_pyfunction!(read, &module)?)?;
    module.add_function(wrap_pyfunction!(write, &module)?)?;
    module.add("__all__", ["read", "write"])?;
    Ok(module)
}

pub(crate) fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_submodule(&json(py, &format!("{name}.json"))?)?
        .finish();
    module.add(
        "StrPath",
        py.get_type::<PyString>().bitor(
            PyModule::import(py, "os")?
                .getattr("PathLike")?
                .get_item(py.get_type::<PyString>())?,
        )?,
    )?;
    module.add("__all__", ["StrPath", "json"])?;
    Ok(module)
}
