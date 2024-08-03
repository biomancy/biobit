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
    let reaper = PyModule::new_bound(parent.py(), &name)?;

    reaper.add_class::<PyReaper>()?;
    reaper.add_class::<PyConfig>()?;
    reaper.add_class::<PyWorkload>()?;
    reaper.add_class::<PyPeak>()?;
    reaper.add_class::<PyHarvest>()?;
    reaper.add_class::<PyHarvestRegion>()?;

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

    cmp::register(&reaper, sysmod)?;
    model::register(&reaper, sysmod)?;
    pcalling::register(&reaper, sysmod)?;
    postfilter::register(&reaper, sysmod)?;

    parent.add_submodule(&reaper)?;
    sysmod.set_item(reaper.name()?, &reaper)?;

    Ok(reaper)
}
