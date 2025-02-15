pub mod overlap;
pub use bits::{Bits, BitsBuilder, PyBits, PyBitsBuilder};

use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

mod bits;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_submodule(&overlap::construct(py, &format!("{name}.overlap"))?)?
        .add_class::<PyBits>()?
        .add_class::<PyBitsBuilder>()?
        .finish();

    Ok(module)
}
