use pyo3::prelude::*;

pub use biobit_core_rs::seqlib::{
    MatesOrientation as RsMatesOrientation, SeqLib as RsSeqLib, Strandedness as RsStrandedness,
};
pub use mates_orientation::MatesOrientation;
pub use strandedness::Strandedness;

mod mates_orientation;
pub mod strandedness;

#[pyclass(eq, ord, hash, frozen)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum SeqLib {
    /// Single-end sequencing library
    Single { strandedness: Strandedness },
    /// Paired-end library
    Paired {
        strandedness: Strandedness,
        orientation: MatesOrientation,
    },
}

impl From<SeqLib> for RsSeqLib {
    fn from(value: SeqLib) -> Self {
        match value {
            SeqLib::Single { strandedness } => RsSeqLib::Single {
                strandedness: strandedness.into(),
            },
            SeqLib::Paired {
                strandedness,
                orientation,
            } => RsSeqLib::Paired {
                strandedness: strandedness.into(),
                orientation: orientation.into(),
            },
        }
    }
}

pub fn register<'a, 'b>(
    name: &'a str,
    parent: &Bound<'b, PyModule>,
    sysmod: &Bound<PyAny>,
) -> PyResult<Bound<'b, PyModule>> {
    let seqlib = PyModule::new_bound(parent.py(), name)?;
    seqlib.add_class::<Strandedness>()?;
    seqlib.add_class::<MatesOrientation>()?;
    seqlib.add_class::<SeqLib>()?;

    // Add the submodule to the parent module & sys.modules cache
    parent.add_submodule(&seqlib)?;
    sysmod.set_item(format!("{}.{}", parent.name()?, name).as_str(), &seqlib)?;

    Ok(seqlib)
}
