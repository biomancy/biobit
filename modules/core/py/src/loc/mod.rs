use pyo3::prelude::*;
use pyo3::PyTypeInfo;

pub use biobit_core_rs::loc::{AsSegment, Contig, Locus, Orientation, Segment, Strand};
pub use locus::{IntoPyLocus, PyLocus};
pub use orientation::{IntoPyOrientation, PyOrientation};
pub use per_orientation::PyPerOrientation;
pub use segment::{IntoPySegment, PySegment};
pub use strand::{IntoPyStrand, PyStrand};

mod locus;
mod orientation;
mod per_orientation;
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
    loc.add_class::<PyPerOrientation>()?;
    loc.add_class::<PySegment>()?;
    loc.add_class::<PyLocus>()?;

    for typbj in [
        PyStrand::type_object_bound(parent.py()),
        PyOrientation::type_object_bound(parent.py()),
        PyPerOrientation::type_object_bound(parent.py()),
        PySegment::type_object_bound(parent.py()),
        PyLocus::type_object_bound(parent.py()),
    ] {
        typbj.setattr("__module__", &name)?
    }

    parent.add_submodule(&loc)?;
    sysmod.set_item(loc.name()?, &loc)?;

    Ok(loc)
}
