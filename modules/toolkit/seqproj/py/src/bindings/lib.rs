use pyo3::prelude::*;

pub use biobit_seqproj_rs::Layout;
pub use layout::PyLayout;

mod layout;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "seqproj";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    PyLayout::__biobit_initialize_complex_enum__(parent.py(), &path, &module)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
