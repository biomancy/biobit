use pyo3::prelude::*;

pub const __VERSION__: &str = env!("CARGO_PKG_VERSION");

#[pymodule]
pub fn _biobit(py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    let sysmod = py.import("sys")?.getattr("modules")?;

    // Core modules
    biobit_core_py::register(module, &sysmod)?;
    biobit_io_py::register(module, &sysmod)?;
    biobit_collections_py::register(module, &sysmod)?;

    // Toolkit
    {
        let toolkit = format!("{}.toolkit", module.name()?);
        let toolkit = PyModule::new(py, &toolkit)?;

        ::biobit_countit_py::register(&toolkit, &sysmod)?;
        ::biobit_reaper_py::register(&toolkit, &sysmod)?;
        ::biobit_seqproj_py::register(&toolkit, &sysmod)?;
        ::biobit_repeto_py::register(&toolkit, &sysmod)?;

        module.add_submodule(&toolkit)?;
        sysmod.set_item(toolkit.name()?, toolkit)?;
    }

    // Constants
    module.add("__version__", __VERSION__)?;

    // Add the module to sys.modules cache
    sysmod.set_item(module.name()?, module)?;

    Ok(())
}
