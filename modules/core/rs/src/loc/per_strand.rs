use derive_getters::Dissolve;
use derive_more::Constructor;

use super::strand::Strand;

/// A struct that holds data for each strand.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Dissolve, Constructor,
)]
pub struct PerStrand<T> {
    pub forward: T,
    pub reverse: T,
}

impl<T> PerStrand<T> {
    /// Gets a reference to the data for the specified strand.
    pub fn get(&self, strand: Strand) -> &T {
        match strand {
            Strand::Forward => &self.forward,
            Strand::Reverse => &self.reverse,
        }
    }

    /// Gets a mutable reference to the data for the specified orientation.
    pub fn get_mut(&mut self, strand: Strand) -> &mut T {
        match strand {
            Strand::Forward => &mut self.forward,
            Strand::Reverse => &mut self.reverse,
        }
    }
}
