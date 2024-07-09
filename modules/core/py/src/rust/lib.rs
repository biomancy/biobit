use pyo3::prelude::*;

pub mod loc;
pub mod seqlib;

pub use biobit_core_rs::LendingIterator;

pub const __VERSION__: &str = env!("CARGO_PKG_VERSION");

#[pymodule]
pub fn _biobit_core_py(py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    let sysmod = py.import_bound("sys")?.getattr("modules")?;

    // Rust submodules
    loc::register("loc", &module, &sysmod)?;
    seqlib::register("seqlib", &module, &sysmod)?;

    // Constants
    module.add("__version__", __VERSION__)?;

    // Add the module to sys.modules cache
    sysmod.set_item(module.name()?, &module)?;

    Ok(())
}
