use pyo3::prelude::*;

pub use biobit_seqproj_rs::Layout;
pub use layout::PyLayout;

mod layout;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.seqproj", parent.name()?);
    let seqlib = PyModule::new_bound(parent.py(), &name)?;

    PyLayout::__biobit_initialize_complex_enum__(parent.py(), &seqlib)?;

    parent.add_submodule(&seqlib)?;
    sysmod.set_item(seqlib.name()?, &seqlib)?;

    Ok(seqlib)
}
