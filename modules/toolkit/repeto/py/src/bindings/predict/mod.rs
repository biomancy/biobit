use biobit_core_py::utils::ImportablePyModuleBuilder;
use biobit_repeto_rs::predict as rs;
pub use filtering::PyFilter;
use pyo3::prelude::*;
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{wrap_pyfunction, Bound, PyResult};
pub use scoring::PyScoring;

use crate::repeats::{PyInvRepeat, PyInvSegment};

mod filtering;
mod scoring;

#[pyfunction]
pub fn run(
    seq: &[u8],
    filter: PyFilter,
    scoring: PyScoring,
) -> PyResult<(Vec<PyInvRepeat>, Vec<i32>)> {
    let filter = filter.into();
    let scoring = scoring.into();

    let (ir, scores) = rs::run(seq, filter, scoring)?;

    // Convert to Py-wrappers
    let ir = Python::with_gil(|py| {
        ir.into_iter()
            .map(|x| {
                let segments = x
                    .segments()
                    .iter()
                    .map(|x| x.cast::<i64>().unwrap())
                    .map(|x| Py::new(py, PyInvSegment::from(x)).unwrap())
                    .collect();
                PyInvRepeat { segments }
            })
            .collect::<Vec<_>>()
    });

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
