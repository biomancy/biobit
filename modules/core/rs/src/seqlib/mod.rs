pub use mates_orientation::MatesOrientation;
pub use strandedness::Strandedness;

mod mates_orientation;
pub mod strandedness;

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
