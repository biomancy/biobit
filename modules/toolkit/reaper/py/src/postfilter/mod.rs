use pyo3::prelude::*;
use pyo3::PyTypeInfo;

pub use nms::PyNMS;

mod nms;

pub fn register<'a>(
    parent: &Bound<'a, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'a, PyModule>> {
    let name = format!("{}.postfilter", parent.name()?);
    let postfilter = PyModule::new_bound(parent.py(), &name)?;

    postfilter.add_class::<PyNMS>()?;
    PyNMS::type_object_bound(parent.py()).setattr("__module__", &name)?;

    parent.add_submodule(&postfilter)?;
    sysmod.set_item(postfilter.name()?, &postfilter)?;

    Ok(postfilter)
}
