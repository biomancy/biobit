use pyo3::prelude::*;

pub use biobit_core_rs::ngs::{Layout, MatesOrientation, Strandedness};
pub use layout::PyLayout;
pub use mates_orientation::{IntoPyMatesOrientation, PyMatesOrientation};
pub use strandedness::{IntoPyStrandness, PyStrandedness};

mod layout;
mod mates_orientation;
mod strandedness;

pub fn register<'b>(
    name: &'_ str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.{}", parent.name()?, name);
    let seqlib = PyModule::new_bound(parent.py(), &name)?;

    seqlib.add_class::<PyStrandedness>()?;
    seqlib.add_class::<PyMatesOrientation>()?;
    seqlib.add_class::<PyLayout>()?;

    parent.add_submodule(&seqlib)?;
    sysmod.set_item(seqlib.name()?, &seqlib)?;

    Ok(seqlib)
}
