use pyo3::prelude::*;
use pyo3::PyTypeInfo;

pub use biobit_core_rs::ngs::{Layout, MatesOrientation, Strandedness};
pub use layout::PyLayout;
pub use mates_orientation::{IntoPyMatesOrientation, PyMatesOrientation};
pub use strandedness::{IntoPyStrandness, PyStrandedness};

mod layout;
mod mates_orientation;
mod strandedness;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "ngs";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    module.add_class::<PyStrandedness>()?;
    module.add_class::<PyMatesOrientation>()?;

    for typbj in [
        PyStrandedness::type_object(parent.py()),
        PyMatesOrientation::type_object(parent.py()),
    ] {
        typbj.setattr("__module__", &path)?
    }

    // Complex enums require special handling
    PyLayout::__biobit_initialize_complex_enum__(parent.py(), &path, &module)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
