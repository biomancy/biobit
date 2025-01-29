pub mod overlap;
pub use bits::{Bits, BitsBuilder, PyBits, PyBitsBuilder};

use pyo3::prelude::*;
use pyo3::PyTypeInfo;

mod bits;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "interval_tree";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    overlap::register(&path, &module, sysmod)?;

    module.add_class::<PyBits>()?;
    module.add_class::<PyBitsBuilder>()?;

    for typbj in [
        PyBits::type_object(parent.py()),
        PyBitsBuilder::type_object(parent.py()),
    ] {
        typbj.setattr("__module__", &path)?
    }

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
