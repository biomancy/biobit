//! Python binding namespace for `bioproj`.
//!
//! The initial implementation intentionally exposes no value types yet. It
//! keeps the module layout ready for bindings as the Rust core grows.

use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

/// Constructs the `bioproj` Python extension module.
pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    Ok(ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .finish())
}
