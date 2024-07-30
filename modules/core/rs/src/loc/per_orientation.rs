use derive_getters::Dissolve;
use derive_more::Constructor;

use super::orientation::Orientation;

/// A struct that holds data for each orientation.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Dissolve, Constructor,
)]
pub struct PerOrientation<T> {
    pub forward: T,
    pub reverse: T,
    pub dual: T,
}

impl<T> PerOrientation<T> {
    /// Gets a reference to the data for the specified orientation.
    pub fn get(&self, orientation: Orientation) -> &T {
        match orientation {
            Orientation::Forward => &self.forward,
            Orientation::Reverse => &self.reverse,
            Orientation::Dual => &self.dual,
        }
    }

    /// Gets a mutable reference to the data for the specified orientation.
    pub fn get_mut(&mut self, orientation: Orientation) -> &mut T {
        match orientation {
            Orientation::Forward => &mut self.forward,
            Orientation::Reverse => &mut self.reverse,
            Orientation::Dual => &mut self.dual,
        }
    }

    /// Gets an iterator over the data for each orientation. Order is forward, reverse, dual.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.forward, &self.reverse, &self.dual].into_iter()
    }

    /// Gets a mutable iterator over the data for each orientation. Order is forward, reverse, dual.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.forward, &mut self.reverse, &mut self.dual].into_iter()
    }

    /// Applies a function to each orientation.
    pub fn apply(&mut self, mut f: impl FnMut(&mut T)) -> &mut Self {
        f(&mut self.forward);
        f(&mut self.reverse);
        f(&mut self.dual);
        self
    }
}

impl<T> IntoIterator for PerOrientation<T> {
    type Item = (T, Orientation);
    type IntoIter = std::array::IntoIter<(T, Orientation), 3>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (self.forward, Orientation::Forward),
            (self.reverse, Orientation::Reverse),
            (self.dual, Orientation::Dual),
        ]
        .into_iter()
    }
}
