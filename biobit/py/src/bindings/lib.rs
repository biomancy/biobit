use pyo3::prelude::*;

pub const __VERSION__: &str = env!("CARGO_PKG_VERSION");

#[pymodule]
pub fn _biobit(py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    let sysmod = py.import_bound("sys")?.getattr("modules")?;

    // Core modules
    biobit_core_py::register("core", module, &sysmod)?;
    biobit_io_py::register("io", module, &sysmod)?;

    // Toolkit
    biobit_countit_py::register("toolkit.countit", module, &sysmod)?;
    biobit_ripper_py::register("toolkit.ripper", module, &sysmod)?;
    biobit_seqproj_py::register("toolkit.seqproj", module, &sysmod)?;

    // Constants
    module.add("__version__", __VERSION__)?;

    // Add the module to sys.modules cache
    sysmod.set_item(module.name()?, module)?;

    Ok(())
}
