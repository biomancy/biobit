use derive_getters::Dissolve;
use derive_more::{Constructor, From};
use std::ops::{Index, IndexMut};

use super::orientation::Orientation;

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
/// A struct that holds data for each orientation.
#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, From, Dissolve, Constructor,
)]
pub struct PerOrientation<T> {
    pub forward: T,
    pub reverse: T,
    pub dual: T,
}

impl<T> PerOrientation<T> {
    /// Creates a new [PerOrientation<T>] by calling a function for each orientation.
    pub fn with_fn(mut f: impl FnMut(Orientation) -> T) -> Self {
        Self {
            forward: f(Orientation::Forward),
            reverse: f(Orientation::Reverse),
            dual: f(Orientation::Dual),
        }
    }

    /// Creates a new [PerOrientation<T>] by calling a fallible function for each orientation.
    pub fn try_with_fn<E>(mut f: impl FnMut(Orientation) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            forward: f(Orientation::Forward)?,
            reverse: f(Orientation::Reverse)?,
            dual: f(Orientation::Dual)?,
        })
    }

    /// Gets an iterator over the data for each orientation.
    pub fn iter(&self) -> impl Iterator<Item = (Orientation, &T)> {
        self.into_iter()
    }

    /// Gets a mutable iterator over the data for each orientation. Order is forward, reverse, dual.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Orientation, &mut T)> {
        self.into_iter()
    }

    /// Applies a function to each orientation.
    pub fn apply(&mut self, mut f: impl FnMut(Orientation, &mut T)) -> &mut Self {
        f(Orientation::Forward, &mut self.forward);
        f(Orientation::Reverse, &mut self.reverse);
        f(Orientation::Dual, &mut self.dual);
        self
    }

    /// Fallible version of `apply`.
    pub fn try_apply<E>(
        &mut self,
        mut f: impl FnMut(Orientation, &mut T) -> Result<(), E>,
    ) -> Result<&mut Self, E> {
        f(Orientation::Forward, &mut self.forward)?;
        f(Orientation::Reverse, &mut self.reverse)?;
        f(Orientation::Dual, &mut self.dual)?;
        Ok(self)
    }

    /// Maps each orientation to a new value.
    pub fn map<U>(self, mut f: impl FnMut(Orientation, T) -> U) -> PerOrientation<U> {
        PerOrientation {
            forward: f(Orientation::Forward, self.forward),
            reverse: f(Orientation::Reverse, self.reverse),
            dual: f(Orientation::Dual, self.dual),
        }
    }

    /// Fallible version of `map`.
    /// Maps each orientation to a new value, returning an error if the function fails.
    pub fn try_map<U, E>(
        self,
        mut f: impl FnMut(Orientation, T) -> Result<U, E>,
    ) -> Result<PerOrientation<U>, E> {
        Ok(PerOrientation {
            forward: f(Orientation::Forward, self.forward)?,
            reverse: f(Orientation::Reverse, self.reverse)?,
            dual: f(Orientation::Dual, self.dual)?,
        })
    }

    /// Creates a new [PerOrientation<T>] with only the forward orientation set and the others as Default.
    pub fn fwd_with_defaults(forward: T) -> Self
    where
        T: Default,
    {
        Self {
            forward,
            reverse: Default::default(),
            dual: Default::default(),
        }
    }

    /// Creates a new [PerOrientation<T>] with only the reverse orientation set and the others as Default.
    pub fn rev_with_defaults(reverse: T) -> Self
    where
        T: Default,
    {
        Self {
            forward: Default::default(),
            reverse,
            dual: Default::default(),
        }
    }

    /// Creates a new [PerOrientation<T>] with only the dual orientation set and the others as Default.
    pub fn dual_with_defaults(dual: T) -> Self
    where
        T: Default,
    {
        Self {
            forward: Default::default(),
            reverse: Default::default(),
            dual,
        }
    }
}

impl<T> Index<Orientation> for PerOrientation<T> {
    type Output = T;

    fn index(&self, index: Orientation) -> &Self::Output {
        match index {
            Orientation::Forward => &self.forward,
            Orientation::Reverse => &self.reverse,
            Orientation::Dual => &self.dual,
        }
    }
}

impl<T> IndexMut<Orientation> for PerOrientation<T> {
    fn index_mut(&mut self, index: Orientation) -> &mut Self::Output {
        match index {
            Orientation::Forward => &mut self.forward,
            Orientation::Reverse => &mut self.reverse,
            Orientation::Dual => &mut self.dual,
        }
    }
}

impl<T> IntoIterator for PerOrientation<T> {
    type Item = (Orientation, T);
    type IntoIter = std::array::IntoIter<(Orientation, T), 3>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (Orientation::Forward, self.forward),
            (Orientation::Reverse, self.reverse),
            (Orientation::Dual, self.dual),
        ]
        .into_iter()
    }
}

impl<'a, T> IntoIterator for &'a PerOrientation<T> {
    type Item = (Orientation, &'a T);
    type IntoIter = std::array::IntoIter<(Orientation, &'a T), 3>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (Orientation::Forward, &self.forward),
            (Orientation::Reverse, &self.reverse),
            (Orientation::Dual, &self.dual),
        ]
        .into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut PerOrientation<T> {
    type Item = (Orientation, &'a mut T);
    type IntoIter = std::array::IntoIter<(Orientation, &'a mut T), 3>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (Orientation::Forward, &mut self.forward),
            (Orientation::Reverse, &mut self.reverse),
            (Orientation::Dual, &mut self.dual),
        ]
        .into_iter()
    }
}
