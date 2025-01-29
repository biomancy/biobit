use pyo3::prelude::*;
use pyo3::{Bound, PyAny, PyResult, PyTypeInfo};

mod chain_map;

pub use chain_map::{ChainMap, PyChainMap};

pub fn register<'b>(
    path: &str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = "mapping";
    let path = format!("{}.{}", path, name);
    let module = PyModule::new(parent.py(), name)?;

    module.add_class::<PyChainMap>()?;

    for typbj in [PyChainMap::type_object(parent.py())] {
        typbj.setattr("__module__", &path)?
    }

    parent.add_submodule(&module)?;
    sysmod.set_item(path, &module)?;

    Ok(module)
}
