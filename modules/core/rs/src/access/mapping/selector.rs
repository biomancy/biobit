//! Borrowed selection independent of storage, locking, and sessions.

use crate::access::Result;

/// Selects a shared reference derived from the supplied value.
///
/// The output lifetime follows the input, rather than the selector. Selection
/// operates on already borrowed data; the caller supplies any storage protection.
pub trait Selector<A: ?Sized> {
    /// The selected value's type.
    type Target: ?Sized;

    /// Selects a shared value while preserving the input borrow's lifetime.
    fn select<'a>(&self, value: &'a A) -> Result<&'a Self::Target>;
}

/// Selects an exclusive reference, in addition to shared selection.
///
/// Shared and mutable selection should identify the same logical field. This is
/// a semantic contract; the reference lifetimes enforce borrowing safety.
pub trait SelectorMut<A: ?Sized>: Selector<A> {
    /// Selects a mutable value while preserving the input borrow's lifetime.
    fn select_mut<'a>(&self, value: &'a mut A) -> Result<&'a mut Self::Target>;
}

/// Adapts an infallible shared closure and, optionally, a mutable closure.
#[derive(Clone)]
pub struct Infallible<R, W = ()> {
    read: R,
    write: W,
}

impl<R> Infallible<R> {
    pub fn new<A: ?Sized, T: ?Sized>(read: R) -> Self
    where
        R: for<'a> Fn(&'a A) -> &'a T,
    {
        Self { read, write: () }
    }
}

impl<R, W> Infallible<R, W> {
    pub fn new_mut<A: ?Sized, T: ?Sized>(read: R, write: W) -> Self
    where
        R: for<'a> Fn(&'a A) -> &'a T,
        W: for<'a> Fn(&'a mut A) -> &'a mut T,
    {
        Self { read, write }
    }
}

impl<A: ?Sized, T: ?Sized, R, W> Selector<A> for Infallible<R, W>
where
    R: for<'a> Fn(&'a A) -> &'a T,
{
    type Target = T;

    fn select<'a>(&self, value: &'a A) -> Result<&'a T> {
        Ok((self.read)(value))
    }
}

impl<A: ?Sized, T: ?Sized, R, W> SelectorMut<A> for Infallible<R, W>
where
    R: for<'a> Fn(&'a A) -> &'a T,
    W: for<'a> Fn(&'a mut A) -> &'a mut T,
{
    fn select_mut<'a>(&self, value: &'a mut A) -> Result<&'a mut T> {
        Ok((self.write)(value))
    }
}

/// Adapts a fallible shared closure and, optionally, a mutable closure.
#[derive(Clone)]
pub struct Fallible<R, W = ()> {
    read: R,
    write: W,
}

impl<R> Fallible<R> {
    pub fn new<A: ?Sized, T: ?Sized>(read: R) -> Self
    where
        R: for<'a> Fn(&'a A) -> Result<&'a T>,
    {
        Self { read, write: () }
    }
}

impl<R, W> Fallible<R, W> {
    pub fn new_mut<A: ?Sized, T: ?Sized>(read: R, write: W) -> Self
    where
        R: for<'a> Fn(&'a A) -> Result<&'a T>,
        W: for<'a> Fn(&'a mut A) -> Result<&'a mut T>,
    {
        Self { read, write }
    }
}

impl<A: ?Sized, T: ?Sized, R, W> Selector<A> for Fallible<R, W>
where
    R: for<'a> Fn(&'a A) -> Result<&'a T>,
{
    type Target = T;

    fn select<'a>(&self, value: &'a A) -> Result<&'a T> {
        (self.read)(value)
    }
}

impl<A: ?Sized, T: ?Sized, R, W> SelectorMut<A> for Fallible<R, W>
where
    R: for<'a> Fn(&'a A) -> Result<&'a T>,
    W: for<'a> Fn(&'a mut A) -> Result<&'a mut T>,
{
    fn select_mut<'a>(&self, value: &'a mut A) -> Result<&'a mut T> {
        (self.write)(value)
    }
}

/// Applies two selectors in order without allocation.
///
/// Composition is clonable when both selectors are clonable. A failure in the
/// first selector skips the second. Mutable selection requires both to support it.
#[derive(Clone)]
pub struct Then<S, N> {
    first: S,
    next: N,
}

impl<S, N> Then<S, N> {
    pub fn new(first: S, next: N) -> Self {
        Self { first, next }
    }
}

impl<A: ?Sized, S, N> Selector<A> for Then<S, N>
where
    S: Selector<A>,
    S::Target: 'static,
    N: Selector<S::Target>,
{
    type Target = N::Target;

    fn select<'a>(&self, value: &'a A) -> Result<&'a Self::Target> {
        self.next.select(self.first.select(value)?)
    }
}

impl<A: ?Sized, S, N> SelectorMut<A> for Then<S, N>
where
    S: SelectorMut<A>,
    S::Target: 'static,
    N: SelectorMut<S::Target>,
{
    fn select_mut<'a>(&self, value: &'a mut A) -> Result<&'a mut Self::Target> {
        self.next.select_mut(self.first.select_mut(value)?)
    }
}
