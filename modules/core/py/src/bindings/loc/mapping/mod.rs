use pyo3::prelude::*;
use pyo3::{Bound, PyAny, PyResult, PyTypeInfo};

mod chain_map;

pub use chain_map::{ChainMap, PyChainMap};

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.mapping", parent.name()?);
    let module = PyModule::new(parent.py(), &name)?;

    module.add_class::<PyChainMap>()?;

    for typbj in [PyChainMap::type_object(parent.py())] {
        typbj.setattr("__module__", &name)?
    }

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
