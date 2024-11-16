pub mod overlap;
pub use bits::{Bits, BitsBuilder, PyBits, PyBitsBuilder};

use pyo3::prelude::*;
use pyo3::PyTypeInfo;

mod bits;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.interval_tree", parent.name()?);
    let module = PyModule::new(parent.py(), &name)?;

    overlap::register(&module, sysmod)?;

    module.add_class::<PyBits>()?;
    module.add_class::<PyBitsBuilder>()?;

    for typbj in [
        PyBits::type_object(parent.py()),
        PyBitsBuilder::type_object(parent.py()),
    ] {
        typbj.setattr("__module__", &name)?
    }

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
