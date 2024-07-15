use pyo3::prelude::*;

pub use biobit_core_rs::loc::{Locus, Orientation, Segment, AsSegment, Strand};
pub use locus::{PyLocus, IntoPyLocus};
pub use orientation::{PyOrientation, IntoPyOrientation};
pub use segment::{PySegment, IntoPySegment};
pub use strand::{PyStrand, IntoPyStrand};

mod locus;
mod orientation;
mod segment;
mod strand;

pub fn register<'b>(
    name: &'_ str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.{}", parent.name()?, name);
    let loc = PyModule::new_bound(parent.py(), &name)?;

    loc.add_class::<PyStrand>()?;
    loc.add_class::<PyOrientation>()?;
    loc.add_class::<PySegment>()?;
    loc.add_class::<PyLocus>()?;

    parent.add_submodule(&loc)?;
    sysmod.set_item(loc.name()?, &loc)?;

    Ok(loc)
}
