use pyo3::{Bound, PyAny, PyResult, PyTypeInfo};
use pyo3::prelude::{PyAnyMethods, PyModule, PyModuleMethods};

pub use inv::{PyInvRepeat, PyInvSegment};

mod inv;


pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.repeats", parent.name()?);
    let repeats = PyModule::new_bound(parent.py(), &name)?;

    repeats.add_class::<PyInvRepeat>()?;
    repeats.add_class::<PyInvSegment>()?;

    for typbj in [
        PyInvRepeat::type_object_bound(parent.py()),
        PyInvSegment::type_object_bound(parent.py()),
    ] {
        typbj.setattr("__module__", &name)?
    }

    parent.add_submodule(&repeats)?;
    sysmod.set_item(repeats.name()?, &repeats)?;

    Ok(repeats)
}