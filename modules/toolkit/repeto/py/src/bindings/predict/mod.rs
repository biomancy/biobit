use pyo3::{Bound, PyAny, PyResult, PyTypeInfo, wrap_pyfunction};
use pyo3::prelude::*;
use pyo3::prelude::{PyAnyMethods, PyModule, PyModuleMethods};

use biobit_repeto_rs::predict as rs;
pub use filtering::PyFilter;
pub use scoring::PyScoring;

use crate::repeats::PyInvRepeat;

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
    let ir = Python::with_gil(|py| ir.into_iter().map(|x| x.into_py(py)).collect::<Vec<_>>());

    Ok((ir, scores))
}

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.predict", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    module.add_class::<PyFilter>()?;
    module.add_class::<PyScoring>()?;

    for typbj in [
        PyFilter::type_object_bound(parent.py()),
        PyScoring::type_object_bound(parent.py()),
    ] {
        typbj.setattr("__module__", &name)?
    }
    module.add_function(wrap_pyfunction!(run, &module)?)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
