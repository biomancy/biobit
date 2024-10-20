use pyo3::prelude::*;
use pyo3::PyTypeInfo;

pub use reaper::PyReaper;
pub use result::{PyHarvest, PyHarvestRegion, PyPeak};
pub use workload::{PyConfig, PyWorkload};

pub mod cmp;
pub mod model;
pub mod pcalling;
mod postfilter;
mod reaper;
mod result;
mod workload;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.reaper", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    module.add_class::<PyReaper>()?;
    module.add_class::<PyConfig>()?;
    module.add_class::<PyWorkload>()?;
    module.add_class::<PyPeak>()?;
    module.add_class::<PyHarvest>()?;
    module.add_class::<PyHarvestRegion>()?;

    for typbj in [
        PyReaper::type_object_bound(parent.py()),
        PyConfig::type_object_bound(parent.py()),
        PyWorkload::type_object_bound(parent.py()),
        PyPeak::type_object_bound(parent.py()),
        PyHarvest::type_object_bound(parent.py()),
        PyHarvestRegion::type_object_bound(parent.py()),
    ] {
        typbj.setattr("__module__", &name)?
    }

    cmp::register(&module, sysmod)?;
    model::register(&module, sysmod)?;
    pcalling::register(&module, sysmod)?;
    postfilter::register(&module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
