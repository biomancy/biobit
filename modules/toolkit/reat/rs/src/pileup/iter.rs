use std::ops::Index;

use biobit_core_rs::num::PrimUInt;

use super::pileup::Pileup;
use crate::dna::{Observed, Reference};

#[derive(Clone, Copy, Debug)]
pub struct Site<'a, T: PrimUInt> {
    offset: usize,
    counts: &'a Pileup<T>,
}

impl<'a, T: PrimUInt> Site<'a, T> {
    #[inline]
    pub(super) fn new(offset: usize, counts: &'a Pileup<T>) -> Self {
        assert!(offset < counts.len(), "site offset {offset} out of bounds");
        Self { offset, counts }
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn a(&self) -> &T {
        &self.counts.a()[self.offset]
    }

    #[inline]
    pub fn c(&self) -> &T {
        &self.counts.c()[self.offset]
    }

    #[inline]
    pub fn g(&self) -> &T {
        &self.counts.g()[self.offset]
    }

    #[inline]
    pub fn t(&self) -> &T {
        &self.counts.t()[self.offset]
    }

    #[inline]
    pub fn n(&self) -> &T {
        &self.counts.n()[self.offset]
    }

    #[inline]
    pub fn deletion(&self) -> &T {
        &self.counts.deletion()[self.offset]
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
        self[Observed::from(reference)]
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

impl<T: PrimUInt> Index<Observed> for Site<'_, T> {
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

#[derive(Clone, Debug)]
pub struct Sites<'a, T: PrimUInt> {
    offset: usize,
    remaining: usize,
    counts: &'a Pileup<T>,
}

impl<'a, T: PrimUInt> Sites<'a, T> {
    #[inline]
    pub(super) fn new(counts: &'a Pileup<T>) -> Self {
        Self {
            offset: 0,
            remaining: counts.len(),
            counts,
        }
    }
}

impl<'a, T: PrimUInt> Iterator for Sites<'a, T> {
    type Item = Site<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let offset = self.offset;
        self.offset += 1;
        self.remaining -= 1;

        Some(Site::new(offset, self.counts))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T: PrimUInt> ExactSizeIterator for Sites<'_, T> {
    fn len(&self) -> usize {
        self.remaining
    }
}
