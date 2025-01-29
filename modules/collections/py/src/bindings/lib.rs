mod bundle;
pub mod interval_tree;

pub use bundle::IntoPyBundle;

use pyo3::prelude::*;

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "collections";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    interval_tree::register(&path, &module, sysmod)?;

    // module.add_class::<PyFilter>()?;
    // module.add_class::<PyScoring>()?;

    // for typbj in [
    //     PyFilter::type_object(parent.py()),
    //     PyScoring::type_object(parent.py()),
    // ] {
    //     typbj.setattr("__module__", &path)?
    // }

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
