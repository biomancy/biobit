use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::ptr::NonNull;

use biobit_core_rs::num::PrimUInt;

use super::Pileup;
use crate::dna::{Observed, Reference};

#[derive(Debug)]
pub struct SiteMut<'a, T: PrimUInt> {
    offset: usize,
    counts: NonNull<Pileup<T>>,
    _marker: PhantomData<&'a mut T>,
}

// SAFETY: `SiteMut` only exposes the fixed pileup offset it was constructed
// with. Sending it to another thread is equivalent to sending mutable access to
// those elements.
unsafe impl<'a, T: PrimUInt + Send> Send for SiteMut<'a, T> {}

// SAFETY: Shared access to `SiteMut` only exposes shared references to its
// fixed elements; mutation still requires `&mut SiteMut`.
unsafe impl<'a, T: PrimUInt + Sync> Sync for SiteMut<'a, T> {}

impl<'a, T: PrimUInt> SiteMut<'a, T> {
    #[inline]
    pub fn new(offset: usize, counts: &'a mut Pileup<T>) -> Self {
        assert!(offset < counts.len(), "site offset {offset} out of bounds");

        // SAFETY: `counts` is borrowed mutably for `'a`, and `offset` was
        // checked against the pileup length above.
        unsafe { Self::from_raw_parts_unchecked(offset, NonNull::from(counts)) }
    }

    #[inline]
    unsafe fn from_raw_parts_unchecked(offset: usize, counts: NonNull<Pileup<T>>) -> Self {
        Self {
            offset,
            counts,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn observed(&self, observed: Observed) -> T {
        self[observed]
    }

    #[inline]
    pub fn observed_mut(&mut self, observed: Observed) -> &mut T {
        &mut self[observed]
    }

    #[inline]
    pub fn a(&self) -> &T {
        // SAFETY: `SiteMut` constructors guarantee that `counts` points to a
        // live pileup and that `offset` is in bounds. `Vec::as_ptr` does not
        // materialize a reference to the backing buffer before returning its
        // base pointer.
        unsafe {
            &*(*std::ptr::addr_of!((*self.counts.as_ptr()).a))
                .as_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn c(&self) -> &T {
        // SAFETY: See `SiteMut::a`.
        unsafe {
            &*(*std::ptr::addr_of!((*self.counts.as_ptr()).c))
                .as_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn g(&self) -> &T {
        // SAFETY: See `SiteMut::a`.
        unsafe {
            &*(*std::ptr::addr_of!((*self.counts.as_ptr()).g))
                .as_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn t(&self) -> &T {
        // SAFETY: See `SiteMut::a`.
        unsafe {
            &*(*std::ptr::addr_of!((*self.counts.as_ptr()).t))
                .as_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn n(&self) -> &T {
        // SAFETY: See `SiteMut::a`.
        unsafe {
            &*(*std::ptr::addr_of!((*self.counts.as_ptr()).n))
                .as_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn deletion(&self) -> &T {
        // SAFETY: See `SiteMut::a`.
        unsafe {
            &*(*std::ptr::addr_of!((*self.counts.as_ptr()).deletion))
                .as_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn a_mut(&mut self) -> &mut T {
        // SAFETY: `SiteMut` only exposes elements at its own offset. For
        // iterator-produced sites, `SitesMut::next` yields each offset at most
        // once, so two live sites cannot expose the same `(observed, offset)`
        // element. `Vec::as_mut_ptr` does not materialize a reference to the
        // backing buffer before returning its base pointer.
        unsafe {
            &mut *(*std::ptr::addr_of_mut!((*self.counts.as_ptr()).a))
                .as_mut_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn c_mut(&mut self) -> &mut T {
        // SAFETY: See `SiteMut::a_mut`.
        unsafe {
            &mut *(*std::ptr::addr_of_mut!((*self.counts.as_ptr()).c))
                .as_mut_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn g_mut(&mut self) -> &mut T {
        // SAFETY: See `SiteMut::a_mut`.
        unsafe {
            &mut *(*std::ptr::addr_of_mut!((*self.counts.as_ptr()).g))
                .as_mut_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn t_mut(&mut self) -> &mut T {
        // SAFETY: See `SiteMut::a_mut`.
        unsafe {
            &mut *(*std::ptr::addr_of_mut!((*self.counts.as_ptr()).t))
                .as_mut_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn n_mut(&mut self) -> &mut T {
        // SAFETY: See `SiteMut::a_mut`.
        unsafe {
            &mut *(*std::ptr::addr_of_mut!((*self.counts.as_ptr()).n))
                .as_mut_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn deletion_mut(&mut self) -> &mut T {
        // SAFETY: See `SiteMut::a_mut`.
        unsafe {
            &mut *(*std::ptr::addr_of_mut!((*self.counts.as_ptr()).deletion))
                .as_mut_ptr()
                .add(self.offset)
        }
    }

    #[inline]
    pub fn coverage(&self) -> T {
        (*self.a())
            .saturating_add(*self.c())
            .saturating_add(*self.g())
            .saturating_add(*self.t())
            .saturating_add(*self.n())
            .saturating_add(*self.deletion())
    }

    #[inline]
    pub fn matches(&self, reference: Reference) -> T {
        match reference {
            Reference::A => *self.a(),
            Reference::C => *self.c(),
            Reference::G => *self.g(),
            Reference::T => *self.t(),
            Reference::N => *self.n(),
        }
    }

    #[inline]
    pub fn mismatches(&self, reference: Reference) -> T {
        match reference {
            Reference::A => (*self.c())
                .saturating_add(*self.g())
                .saturating_add(*self.t())
                .saturating_add(*self.n())
                .saturating_add(*self.deletion()),
            Reference::C => (*self.a())
                .saturating_add(*self.g())
                .saturating_add(*self.t())
                .saturating_add(*self.n())
                .saturating_add(*self.deletion()),
            Reference::G => (*self.a())
                .saturating_add(*self.c())
                .saturating_add(*self.t())
                .saturating_add(*self.n())
                .saturating_add(*self.deletion()),
            Reference::T => (*self.a())
                .saturating_add(*self.c())
                .saturating_add(*self.g())
                .saturating_add(*self.n())
                .saturating_add(*self.deletion()),
            Reference::N => (*self.a())
                .saturating_add(*self.c())
                .saturating_add(*self.g())
                .saturating_add(*self.t())
                .saturating_add(*self.deletion()),
        }
    }

    #[inline]
    pub fn is_covered(&self) -> bool {
        self.coverage() > T::zero()
    }
}

impl<T: PrimUInt> Index<Observed> for SiteMut<'_, T> {
    type Output = T;

    #[inline]
    fn index(&self, index: Observed) -> &Self::Output {
        match index {
            Observed::A => self.a(),
            Observed::C => self.c(),
            Observed::G => self.g(),
            Observed::T => self.t(),
            Observed::N => self.n(),
            Observed::Deletion => self.deletion(),
        }
    }
}

impl<T: PrimUInt> IndexMut<Observed> for SiteMut<'_, T> {
    #[inline]
    fn index_mut(&mut self, index: Observed) -> &mut Self::Output {
        match index {
            Observed::A => self.a_mut(),
            Observed::C => self.c_mut(),
            Observed::G => self.g_mut(),
            Observed::T => self.t_mut(),
            Observed::N => self.n_mut(),
            Observed::Deletion => self.deletion_mut(),
        }
    }
}

#[derive(Debug)]
pub struct SitesMut<'a, T: PrimUInt> {
    offset: usize,
    remaining: usize,
    counts: NonNull<Pileup<T>>,
    _marker: PhantomData<&'a mut Pileup<T>>,
}

// SAFETY: `SitesMut` owns the exclusive pileup borrow for `'a` and yields each
// offset at most once, so sending it preserves the same requirements as sending
// mutable access to the underlying elements.
unsafe impl<'a, T: PrimUInt + Send> Send for SitesMut<'a, T> {}

// SAFETY: Shared access to `SitesMut` does not expose mutable elements or
// advance iteration; `next` still requires `&mut SitesMut`.
unsafe impl<'a, T: PrimUInt + Sync> Sync for SitesMut<'a, T> {}

impl<'a, T: PrimUInt> SitesMut<'a, T> {
    #[inline]
    pub(super) fn new(counts: &'a mut Pileup<T>) -> Self {
        let remaining = counts.len();

        Self {
            offset: 0,
            remaining,
            counts: NonNull::from(counts),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: PrimUInt> Iterator for SitesMut<'a, T> {
    type Item = SiteMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let offset = self.offset;
        self.offset += 1;
        self.remaining -= 1;

        // SAFETY: `SitesMut` owns the mutable borrow of the pileup for `'a`,
        // `remaining` is initialized from `counts.len()`, and `next` yields each
        // offset at most once. Each yielded `SiteMut` can only access elements
        // at its offset, so simultaneously live sites are disjoint.
        let site = unsafe { SiteMut::from_raw_parts_unchecked(offset, self.counts) };
        Some(site)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T: PrimUInt> ExactSizeIterator for SitesMut<'_, T> {
    fn len(&self) -> usize {
        self.remaining
    }
}
