use eyre::Result;
use pyo3::{Bound, PyAny, PyResult};
use pyo3::prelude::*;
use pyo3::prelude::{PyAnyMethods, PyModule, PyModuleMethods};

use biobit_repeto_rs::repeats::InvRepeat;

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
                    .map(|s| s.borrow(py).rs().clone())
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
    return Ok((ir, total_score));
}

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.optimize", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    module.add_function(wrap_pyfunction!(run, &module)?)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
