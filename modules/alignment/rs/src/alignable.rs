// Instead of making a custom trait here I must support Rust builtin traits for containers
// once they are ready: https://internals.rust-lang.org/t/traits-that-should-be-in-std-but-arent/3002

use derive_getters::Dissolve;
use derive_more::Constructor;

/// Trait for types that can be aligned.
pub trait Alignable {
    /// The type of individual symbols/elements being aligned.
    type Symbol;

    /// Returns true if the object is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the object.
    fn len(&self) -> usize;

    /// Returns the symbol at the given position.
    fn at(&self, pos: usize) -> &Self::Symbol;

    /// Return the reversed version of the alignable object.
    fn reversed(&self) -> Reversed<&Self>
    where
        Self: Sized,
    {
        Reversed::new(self)
    }
}

impl<A: Alignable + ?Sized> Alignable for &A {
    type Symbol = A::Symbol;

    #[inline]
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }

    #[inline]
    fn len(&self) -> usize {
        (**self).len()
    }

    #[inline]
    fn at(&self, pos: usize) -> &Self::Symbol {
        (**self).at(pos)
    }
}

impl<T: Copy> Alignable for &[T] {
    type Symbol = T;

    #[inline(always)]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    #[inline(always)]
    fn at(&self, pos: usize) -> &Self::Symbol {
        &self[pos]
    }
}

impl<T: Copy> Alignable for Vec<T> {
    type Symbol = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn at(&self, pos: usize) -> &Self::Symbol {
        &self[pos]
    }
}

/// A sequence viewed in reverse order. Pass a slice or shared reference to borrow its symbols.
#[derive(Dissolve, Constructor, Debug, Clone, PartialEq, Eq, Hash)]
// derive_more::Constructor generates explicit field names.
#[allow(clippy::redundant_field_names)]
pub struct Reversed<T: Alignable> {
    base: T,
}

impl<T: Alignable> Alignable for Reversed<T> {
    type Symbol = T::Symbol;

    /// Returns true if the reversed object is empty.
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.base.is_empty()
    }

    /// Total length of the reversed object.
    #[inline(always)]
    fn len(&self) -> usize {
        self.base.len()
    }

    /// Returns the symbol at the given position in the reversed object.
    #[inline(always)]
    fn at(&self, pos: usize) -> &Self::Symbol {
        self.base.at(self.base.len() - pos - 1)
    }
}
