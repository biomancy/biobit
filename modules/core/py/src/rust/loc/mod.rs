use pyo3::prelude::*;

pub use biobit_core_rs::loc::{
    Locus as RsLocus, Orientation as RsOrientation, Segment as RsSegment, Strand as RsStrand,
};
pub use locus::Locus;
pub use orientation::Orientation;
pub use segment::Segment;
pub use strand::Strand;

mod locus;
mod orientation;
mod segment;
mod strand;

pub fn register<'a, 'b>(
    name: &'a str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let loc = PyModule::new_bound(parent.py(), name)?;
    loc.add_class::<Strand>()?;
    loc.add_class::<Orientation>()?;
    loc.add_class::<Segment>()?;
    loc.add_class::<Locus>()?;

    // Add the submodule to the parent module & sys.modules cache
    parent.add_submodule(&loc)?;
    sysmod.set_item(format!("{}.{}", parent.name()?, name).as_str(), &loc)?;

    Ok(loc)
}
