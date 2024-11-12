use pyo3::prelude::*;
use pyo3::PyTypeInfo;

pub use biobit_core_rs::loc::{Contig, Interval, IntervalOp, Locus, Orientation, Strand};
pub use interval::{IntoPyInterval, PyInterval};
pub use locus::{IntoPyLocus, PyLocus};
pub use orientation::{IntoPyOrientation, PyOrientation};
pub use per_orientation::PyPerOrientation;
pub use per_strand::PyPerStrand;
pub use strand::{IntoPyStrand, PyStrand};

mod interval;
mod locus;
mod orientation;
mod per_orientation;
mod per_strand;
mod strand;

pub fn register<'b>(
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let name = format!("{}.loc", parent.name()?);
    let module = PyModule::new_bound(parent.py(), &name)?;

    module.add_class::<PyStrand>()?;
    module.add_class::<PyOrientation>()?;
    module.add_class::<PyPerOrientation>()?;
    module.add_class::<PyPerStrand>()?;
    module.add_class::<PyInterval>()?;
    module.add_class::<PyLocus>()?;

    for typbj in [
        PyStrand::type_object_bound(parent.py()),
        PyOrientation::type_object_bound(parent.py()),
        PyPerOrientation::type_object_bound(parent.py()),
        PyPerStrand::type_object_bound(parent.py()),
        PyInterval::type_object_bound(parent.py()),
        PyLocus::type_object_bound(parent.py()),
    ] {
        typbj.setattr("__module__", &name)?
    }

    parent.add_submodule(&module)?;
    sysmod.set_item(module.name()?, &module)?;

    Ok(module)
}
