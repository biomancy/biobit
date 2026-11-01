use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult};

use crate::repeats::PyInvRepeat;

#[pyfunction]
pub fn run(
    py: Python<'_>,
    ir: Vec<Py<PyInvRepeat>>,
    scores: Vec<i64>,
) -> PyResult<(Vec<Py<PyInvRepeat>>, i64)> {
    let rs = ir
        .iter()
        .map(|repeat| repeat.borrow(py).rs.clone())
        .collect::<Vec<_>>();
    let (solution, total_score) =
        py.detach(move || ::biobit_repeto_rs::optimize::run(&rs, &scores))?;

    // Shallow copy solution repeats
    let ir = solution.into_iter().map(|x| ir[x].clone_ref(py)).collect();

    // Return the result
    Ok((ir, total_score))
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .finish();
    module.add_function(wrap_pyfunction!(run, &module)?)?;

    Ok(module)
}
