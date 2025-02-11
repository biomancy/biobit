use pyo3::prelude::{PyAnyMethods, PyModule, PyModuleMethods};
use pyo3::{Bound, PyAny, PyResult, PyTypeInfo};

pub use inv::{PyInvRepeat, PyInvSegment};

mod inv;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "repeats";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    module.add_class::<PyInvRepeat>()?;
    module.add_class::<PyInvSegment>()?;

    for typbj in [
        PyInvRepeat::type_object(parent.py()),
        PyInvSegment::type_object(parent.py()),
    ] {
        typbj.setattr("__module__", &path)?
    }

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
