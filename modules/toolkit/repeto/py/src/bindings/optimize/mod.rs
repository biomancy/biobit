use biobit_core_py::utils::ImportablePyModuleBuilder;
use biobit_repeto_rs::repeats::InvRepeat;
use eyre::Result;
use pyo3::prelude::*;
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult};

use crate::repeats::PyInvRepeat;

#[pyfunction]
pub fn run(ir: Vec<Py<PyInvRepeat>>, scores: Vec<i64>) -> PyResult<(Vec<Py<PyInvRepeat>>, i64)> {
    // Transform to an optimized Rust representation
    let rs = Python::with_gil(|py| {
        ir.iter()
            .map(|x| {
                let segments = x
                    .borrow(py)
                    .segments
                    .iter()
                    .map(|s| *s.borrow(py).rs())
                    .collect();
                InvRepeat::new(segments)
            })
            .collect::<Result<Vec<_>>>()
    })?;

    // Run the solution
    let (solution, total_score) = ::biobit_repeto_rs::optimize::run(&rs, &scores)?;

    // Shallow copy solution repeats
    let ir = Python::with_gil(|py| solution.into_iter().map(|x| ir[x].clone_ref(py)).collect());

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
