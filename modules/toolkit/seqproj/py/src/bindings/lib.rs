use biobit_core_py::utils::ImportablePyModuleBuilder;
pub use biobit_seqproj_rs::Layout;
pub use layout::PyLayout;
use pyo3::prelude::*;

mod layout;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?.defaults()?;
    let module = layout::__biobit_initialize_complex_enum__(module)?.finish();

    Ok(module)
}
