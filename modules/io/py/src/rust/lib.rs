use pyo3::prelude::*;

pub mod bam;

pub const __VERSION__: &str = env!("CARGO_PKG_VERSION");

#[pymodule]
pub fn _biobit_io_py(py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    let sysmod = py.import_bound("sys")?.getattr("modules")?;

    // Rust submodules
    bam::register("bam", &module, &sysmod)?;

    // Constants
    module.add("__version__", __VERSION__)?;

    // Add the module to sys.modules cache
    sysmod.set_item(module.name()?, &module)?;

    Ok(())
}
