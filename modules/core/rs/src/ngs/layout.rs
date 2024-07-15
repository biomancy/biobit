use super::{mates_orientation::MatesOrientation, strandedness::Strandedness};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Layout {
    /// Each source molecule is sequenced only once (single-end sequencing) in one direction.
    Single { strandedness: Strandedness },
    /// Each source molecule is sequenced twice (paired-end sequencing) following a known orientation.
    /// Strandedness interpretation is orientation-dependent.
    Paired {
        strandedness: Strandedness,
        orientation: MatesOrientation,
    },
}
