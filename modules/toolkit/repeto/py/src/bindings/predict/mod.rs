use biobit_core_py::utils::ImportablePyModuleBuilder;
use biobit_repeto_rs::predict as rs;
use eyre::{OptionExt, Result};
pub use filtering::PyFilter;
use pyo3::prelude::*;
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult, wrap_pyfunction};
pub use scoring::PyScoring;

use crate::repeats::PyInvRepeat;

mod filtering;
mod scoring;

#[pyfunction]
pub fn run(
    seq: &[u8],
    filter: PyFilter,
    scoring: PyScoring,
) -> Result<(Vec<PyInvRepeat>, Vec<i32>)> {
    let filter = filter.into();
    let scoring = scoring.into();

    let (ir, scores) = rs::run(seq, filter, scoring)?;

    // Convert to Py-wrappers
    let ir = ir
        .into_iter()
        .map(|x| x.cast().map(PyInvRepeat::from))
        .collect::<Option<Vec<_>>>()
        .ok_or_eyre("Failed to cast IR from usize to i64")?;

    Ok((ir, scores))
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyFilter>()?
        .add_class::<PyScoring>()?
        .finish();
    module.add_function(wrap_pyfunction!(run, &module)?)?;

    Ok(module)
}
