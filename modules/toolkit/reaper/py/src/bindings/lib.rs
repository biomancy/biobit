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
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "reaper";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    module.add_class::<PyReaper>()?;
    module.add_class::<PyConfig>()?;
    module.add_class::<PyWorkload>()?;
    module.add_class::<PyPeak>()?;
    module.add_class::<PyHarvest>()?;
    module.add_class::<PyHarvestRegion>()?;

    for typbj in [
        PyReaper::type_object(parent.py()),
        PyConfig::type_object(parent.py()),
        PyWorkload::type_object(parent.py()),
        PyPeak::type_object(parent.py()),
        PyHarvest::type_object(parent.py()),
        PyHarvestRegion::type_object(parent.py()),
    ] {
        typbj.setattr("__module__", &path)?
    }

    cmp::register(&path, &module, sysmod)?;
    model::register(&path, &module, sysmod)?;
    pcalling::register(&path, &module, sysmod)?;
    postfilter::register(&path, &module, sysmod)?;

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
