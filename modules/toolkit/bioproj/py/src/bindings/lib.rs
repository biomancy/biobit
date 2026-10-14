use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyString};

mod data;
mod design;
mod io;
mod primitives;
mod project;
mod provenance;
mod utils;

pub use biobit_bioproj_rs::{Layout, MatesOrientation, Strandedness};
pub use primitives::{
    IntoPyMatesOrientation, IntoPyStrandedness, PyLayout, PyMatesOrientation, PyStrandedness,
};

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?.defaults()?;
    let module = primitives::register(py, module)?
        .add_class::<project::PyProjectId>()?
        .add_class::<project::PyProject>()?
        .add_submodule(&provenance::construct(py, &format!("{name}.provenance"))?)?
        .add_submodule(&data::construct(py, &format!("{name}.data"))?)?
        .add_submodule(&design::construct(py, &format!("{name}.design"))?)?
        .add_submodule(&io::construct(py, &format!("{name}.io"))?)?
        .finish();
    module.add(
        "MetaValue",
        py.get_type::<PyString>().bitor(py.get_type::<PyBool>())?,
    )?;
    let literal = PyModule::import(py, "typing")?.getattr("Literal")?;
    module.add(
        "IntoStrandedness",
        py.get_type::<PyStrandedness>().bitor(literal.get_item((
            "Forward",
            "Reverse",
            "Unstranded",
        ))?)?,
    )?;
    module.add(
        "IntoMatesOrientation",
        py.get_type::<PyMatesOrientation>()
            .bitor(literal.get_item("Inward")?)?,
    )?;
    module.add(
        "__all__",
        [
            "FilePath",
            "IntoMatesOrientation",
            "IntoStrandedness",
            "Layout",
            "MatesOrientation",
            "MetaValue",
            "Project",
            "ProjectId",
            "Strandedness",
            "data",
            "design",
            "io",
            "provenance",
        ],
    )?;
    Ok(module)
}
