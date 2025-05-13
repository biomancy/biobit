use pyo3::prelude::*;

use crate::utils::ImportablePyModuleBuilder;
pub use biobit_core_rs::{LendingIterator, num, parallelism, source};

pub mod loc;
pub mod ngs;
pub mod pickle;
pub mod utils;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_submodule(&loc::construct(py, &format!("{name}.loc"))?)?
        .add_submodule(&ngs::construct(py, &format!("{name}.ngs"))?)?
        .finish();

    Ok(module)
}
