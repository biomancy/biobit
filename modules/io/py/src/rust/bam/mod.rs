use pyo3::prelude::*;

pub use biobit_io_rs::bam::{
    AdaptersForIndexedBAM, AlignmentSegments as RsAlignmentSegments, IndexedBAM, Reader as RsReader,
};
pub use reader::Reader;

mod reader;

pub fn register<'a, 'b>(
    name: &'a str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let bam = PyModule::new_bound(parent.py(), name)?;
    bam.add_class::<Reader>()?;

    // Add the submodule to the parent module & sys.modules cache
    parent.add_submodule(&bam)?;
    sysmod.set_item(format!("{}.{}", parent.name()?, name).as_str(), &bam)?;

    Ok(bam)
}
