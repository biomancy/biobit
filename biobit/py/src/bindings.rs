use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;

pub const __VERSION__: &str = env!("CARGO_PKG_VERSION");

#[pymodule]
pub fn rs(py: Python, module: Bound<'_, PyModule>) -> PyResult<()> {
    let name = module.name()?.extract::<String>()?;
    let module = ImportablePyModuleBuilder::from(module)
        .defaults()?
        // Core modules
        .add_submodule(&biobit_core_py::construct(py, &format!("{name}.core"))?)?
        .add_submodule(&biobit_io_py::construct(py, &format!("{name}.io"))?)?
        .add_submodule(&biobit_collections_py::construct(
            py,
            &format!("{name}.collections"),
        )?)?;

    // Toolkit
    let name = format!("{name}.toolkit");
    let toolkit = ImportablePyModuleBuilder::new(py, &name)?
        .defaults()?
        .add_submodule(&biobit_countit_py::construct(
            py,
            &format!("{name}.countit"),
        )?)?
        .add_submodule(&biobit_reaper_py::construct(py, &format!("{name}.reaper"))?)?
        .add_submodule(&biobit_seqproj_py::construct(
            py,
            &format!("{name}.seqproj"),
        )?)?
        .add_submodule(&biobit_repeto_py::construct(py, &format!("{name}.repeto"))?)?
        .finish();

    let module = module.add_submodule(&toolkit)?.finish();

    // Constants
    module.add("__version__", __VERSION__)?;

    Ok(())
}
