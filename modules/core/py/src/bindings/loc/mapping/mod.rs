use pyo3::prelude::*;
use pyo3::{Bound, PyResult};

mod chain_map;

use crate::utils::ImportablePyModuleBuilder;
pub use chain_map::{ChainMap, PyChainMap};

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyChainMap>()?
        .finish();

    Ok(module)
}
