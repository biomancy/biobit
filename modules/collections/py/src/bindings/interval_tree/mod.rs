pub use bits::{PyBits, PyBitsBuilder};
pub use hits::{PyBatchHits, PyHits};
pub use segments::{PyBatchHitSegments, PyHitSegments};

use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

mod bits;
mod hits;
mod segments;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyBits>()?
        .add_class::<PyBitsBuilder>()?
        .add_class::<PyHits>()?
        .add_class::<PyBatchHits>()?
        .add_class::<PyHitSegments>()?
        .add_class::<PyBatchHitSegments>()?
        .finish();

    Ok(module)
}
