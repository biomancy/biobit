use derive_getters::Dissolve;
use derive_more::Constructor;

use super::strand::Strand;

/// A struct that holds data for each orientation.
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

    /// Gets an iterator over the data for each orientation.
    pub fn iter(&self) -> impl Iterator<Item = (Strand, &T)> {
        self.into_iter()
    }

    /// Gets a mutable iterator over the data for each orientation. Order is forward, reverse, dual.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Strand, &mut T)> {
        self.into_iter()
    }

    /// Applies a function to each orientation.
    pub fn apply(&mut self, mut f: impl FnMut(Strand, &mut T)) -> &mut Self {
        f(Strand::Forward, &mut self.forward);
        f(Strand::Reverse, &mut self.reverse);
        self
    }

    /// Fallible version of `apply`.
    pub fn try_apply<E>(
        &mut self,
        mut f: impl FnMut(Strand, &mut T) -> Result<(), E>,
    ) -> Result<&mut Self, E> {
        f(Strand::Forward, &mut self.forward)?;
        f(Strand::Reverse, &mut self.reverse)?;
        Ok(self)
    }

    /// Maps each orientation to a new value.
    pub fn map<U>(self, mut f: impl FnMut(Strand, T) -> U) -> PerStrand<U> {
        PerStrand {
            forward: f(Strand::Forward, self.forward),
            reverse: f(Strand::Reverse, self.reverse),
        }
    }

    /// Fallible version of `map`.
    /// Maps each orientation to a new value, returning an error if the function fails.
    pub fn try_map<U, E>(
        self,
        mut f: impl FnMut(Strand, T) -> Result<U, E>,
    ) -> Result<PerStrand<U>, E> {
        Ok(PerStrand {
            forward: f(Strand::Forward, self.forward)?,
            reverse: f(Strand::Reverse, self.reverse)?,
        })
    }
}

impl<T> IntoIterator for PerStrand<T> {
    type Item = (Strand, T);
    type IntoIter = std::array::IntoIter<(Strand, T), 2>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (Strand::Forward, self.forward),
            (Strand::Reverse, self.reverse),
        ]
        .into_iter()
    }
}

impl<'a, T> IntoIterator for &'a PerStrand<T> {
    type Item = (Strand, &'a T);
    type IntoIter = std::array::IntoIter<(Strand, &'a T), 2>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (Strand::Forward, &self.forward),
            (Strand::Reverse, &self.reverse),
        ]
        .into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut PerStrand<T> {
    type Item = (Strand, &'a mut T);
    type IntoIter = std::array::IntoIter<(Strand, &'a mut T), 2>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (Strand::Forward, &mut self.forward),
            (Strand::Reverse, &mut self.reverse),
        ]
        .into_iter()
    }
}
