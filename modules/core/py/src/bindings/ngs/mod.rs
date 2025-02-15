use pyo3::prelude::*;

use crate::utils::ImportablePyModuleBuilder;
pub use biobit_core_rs::ngs::{Layout, MatesOrientation, Strandedness};
pub use layout::PyLayout;
pub use mates_orientation::{IntoPyMatesOrientation, PyMatesOrientation};
pub use strandedness::{IntoPyStrandness, PyStrandedness};

mod layout;
mod mates_orientation;
mod strandedness;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyStrandedness>()?
        .add_class::<PyMatesOrientation>()?;
    let module = layout::__biobit_initialize_complex_enum__(module)?.finish();

    Ok(module)
}
