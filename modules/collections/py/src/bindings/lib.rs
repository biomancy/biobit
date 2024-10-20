mod bundle;
pub mod interval_tree;

pub use bundle::IntoPyBundle;

use pyo3::prelude::*;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.collections", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    interval_tree::register(&module, sysmod)?;

    // module.add_class::<PyFilter>()?;
    // module.add_class::<PyScoring>()?;

    // for typbj in [
    //     PyFilter::type_object_bound(parent.py()),
    //     PyScoring::type_object_bound(parent.py()),
    // ] {
    //     typbj.setattr("__module__", &name)?
    // }

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
