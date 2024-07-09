mod countit;

use pyo3::prelude::*;

pub const __VERSION__: &str = env!("CARGO_PKG_VERSION");

#[pyfunction]
pub fn run() -> PyResult<()> {
    println!("Hello from Rust!");
    Ok(())
}

#[pymodule]
pub fn _biobit_countit_py(py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    let sysmod = py.import_bound("sys")?.getattr("modules")?;

    // Rust submodules
    // bam::register("bam", &module, &sysmod)?;
    module.add_function(wrap_pyfunction!(run, module)?)?;

    // Constants
    module.add("__version__", __VERSION__)?;

    // Add the module to sys.modules cache
    sysmod.set_item(module.name()?, &module)?;

    Ok(())
}
