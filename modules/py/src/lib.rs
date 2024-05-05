use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "_rs")]
fn string_sum(_: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
