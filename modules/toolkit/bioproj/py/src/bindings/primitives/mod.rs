//! Shared primitive bindings and conversions used by the project model.

mod entity_family;
mod file_path;
mod layout;
mod mates_orientation;
mod meta;
mod strandedness;
mod typed_id;

use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
use pyo3::types::PyMapping;

use layout::{PyLayoutPaired, PyLayoutSingle};

pub(crate) use entity_family::define_family_conversions;
pub use file_path::PyFilePath;
pub use layout::PyLayout;
pub use mates_orientation::{IntoPyMatesOrientation, PyMatesOrientation};
pub use meta::PyMeta;
pub use strandedness::{IntoPyStrandedness, PyStrandedness};
pub(crate) use typed_id::define_id_binding;

pub fn register<'py>(
    py: Python<'py>,
    module: ImportablePyModuleBuilder<'py>,
) -> PyResult<ImportablePyModuleBuilder<'py>> {
    PyMapping::register::<PyMeta>(py)?;
    let module = module
        .add_class::<PyStrandedness>()?
        .add_class::<PyMatesOrientation>()?
        .add_class::<PyLayout>()?
        .add_class::<PyLayoutSingle>()?
        .add_class::<PyLayoutPaired>()?
        .add_class::<PyMeta>()?
        .add_class::<PyFilePath>()?
        .finish();

    // Expose the concrete classes under Layout and retain their module aliases.
    // Nested qualnames let pickle resolve instances through Layout.Single/Paired.
    for (name, variant) in [
        ("Single", py.get_type::<PyLayoutSingle>()),
        ("Paired", py.get_type::<PyLayoutPaired>()),
    ] {
        variant.setattr("__qualname__", format!("Layout.{name}"))?;
        py.get_type::<PyLayout>().setattr(name, variant)?;
    }
    Ok(ImportablePyModuleBuilder::from(module))
}
