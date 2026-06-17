use std::ops::{Index, IndexMut};

use biobit_core_rs::num::PrimUInt;
#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::Dissolve;
use eyre::{Result, ensure};

use super::iter::{Site, Sites};
use crate::dna::Observed;

pub use iter_mut::{SiteMut, SitesMut};

#[path = "iter_mut.rs"]
mod iter_mut;

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, Eq, PartialEq, Debug, Default, Dissolve)]
pub struct Pileup<T> {
    a: Vec<T>,
    c: Vec<T>,
    g: Vec<T>,
    t: Vec<T>,
    n: Vec<T>,
    deletion: Vec<T>,
}

impl<T: PrimUInt> Pileup<T> {
    pub fn new(
        a: Vec<T>,
        c: Vec<T>,
        g: Vec<T>,
        t: Vec<T>,
        n: Vec<T>,
        deletion: Vec<T>,
    ) -> Result<Self> {
        let len = a.len();
        ensure!(c.len() == len, "C counts length does not match A counts");
        ensure!(g.len() == len, "G counts length does not match A counts");
        ensure!(t.len() == len, "T counts length does not match A counts");
        ensure!(n.len() == len, "N counts length does not match A counts");
        ensure!(
            deletion.len() == len,
            "Deletion counts length does not match A counts"
        );
        Ok(Self {
            a,
            c,
            g,
            t,
            n,
            deletion,
        })
    }

    #[inline]
    pub fn zeros(len: usize) -> Self {
        Self {
            a: vec![T::default(); len],
            c: vec![T::default(); len],
            g: vec![T::default(); len],
            t: vec![T::default(); len],
            n: vec![T::default(); len],
            deletion: vec![T::default(); len],
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            a: Vec::with_capacity(capacity),
            c: Vec::with_capacity(capacity),
            g: Vec::with_capacity(capacity),
            t: Vec::with_capacity(capacity),
            n: Vec::with_capacity(capacity),
            deletion: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.a.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.a.capacity()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.a.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> Sites<'_, T> {
        Sites::new(self)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> SitesMut<'_, T> {
        SitesMut::new(self)
    }

    #[inline]
    pub fn site(&self, offset: usize) -> Site<'_, T> {
        Site::new(offset, self)
    }

    #[inline]
    pub fn site_mut(&mut self, offset: usize) -> SiteMut<'_, T> {
        SiteMut::new(offset, self)
    }

    #[inline]
    pub fn a(&self) -> &[T] {
        &self.a
    }

    #[inline]
    pub fn c(&self) -> &[T] {
        &self.c
    }

    #[inline]
    pub fn g(&self) -> &[T] {
        &self.g
    }

    #[inline]
    pub fn t(&self) -> &[T] {
        &self.t
    }

    #[inline]
    pub fn n(&self) -> &[T] {
        &self.n
    }

    #[inline]
    pub fn deletion(&self) -> &[T] {
        &self.deletion
    }

    #[inline]
    pub fn a_mut(&mut self) -> &mut [T] {
        &mut self.a
    }

    #[inline]
    pub fn c_mut(&mut self) -> &mut [T] {
        &mut self.c
    }

    #[inline]
    pub fn g_mut(&mut self) -> &mut [T] {
        &mut self.g
    }

    #[inline]
    pub fn t_mut(&mut self) -> &mut [T] {
        &mut self.t
    }

    #[inline]
    pub fn n_mut(&mut self) -> &mut [T] {
        &mut self.n
    }

    #[inline]
    pub fn deletion_mut(&mut self) -> &mut [T] {
        &mut self.deletion
    }

    #[inline]
    pub fn reset(&mut self, len: usize) {
        for array in [
            &mut self.a,
            &mut self.c,
            &mut self.g,
            &mut self.t,
            &mut self.n,
            &mut self.deletion,
        ] {
            array.clear();
            array.resize(len, T::zero());
        }
    }
}

impl<T: PrimUInt> Index<Observed> for Pileup<T> {
    type Output = [T];

    fn index(&self, index: Observed) -> &Self::Output {
        match index {
            Observed::A => &self.a,
            Observed::C => &self.c,
            Observed::G => &self.g,
            Observed::T => &self.t,
            Observed::N => &self.n,
            Observed::Deletion => &self.deletion,
        }
    }
}

impl<T: PrimUInt> IndexMut<Observed> for Pileup<T> {
    fn index_mut(&mut self, index: Observed) -> &mut Self::Output {
        match index {
            Observed::A => &mut self.a,
            Observed::C => &mut self.c,
            Observed::G => &mut self.g,
            Observed::T => &mut self.t,
            Observed::N => &mut self.n,
            Observed::Deletion => &mut self.deletion,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::Reference;

    #[test]
    fn zeros() {
        let pileup = Pileup::<u32>::zeros(3);
        assert_eq!(pileup.len(), 3);
        assert_eq!(&pileup[Observed::A], &[0, 0, 0]);
        assert_eq!(&pileup[Observed::Deletion], &[0, 0, 0]);
        assert!(!pileup.is_empty());
    }

    #[test]
    fn with_capacity_starts_empty() {
        let pileup = Pileup::<u32>::with_capacity(8);
        assert_eq!(pileup.len(), 0);
        assert!(pileup.is_empty());
        assert!(pileup.capacity() >= 8);
    }

    #[test]
    fn new_validates_lengths() -> Result<()> {
        let pileup = Pileup::<u32>::new(vec![1], vec![2], vec![3], vec![4], vec![5], vec![6])?;
        assert_eq!(pileup.len(), 1);
        assert_eq!(pileup[Observed::A][0], 1);
        assert_eq!(pileup[Observed::Deletion][0], 6);
        Ok(())
    }

    #[test]
    fn new_errors_on_mismatched_lengths() {
        assert!(
            Pileup::<u32>::new(vec![0], vec![0, 0], vec![0], vec![0], vec![0], vec![0],).is_err()
        );
    }

    #[test]
    fn index_mut() {
        let mut pileup = Pileup::<u32>::zeros(2);
        pileup[Observed::Deletion][1] = 3;
        assert_eq!(pileup.deletion(), &[0, 3]);
    }

    #[test]
    fn field_mut_accessors() {
        let mut pileup = Pileup::<u32>::zeros(2);

        pileup.a_mut()[0] = 1;
        pileup.c_mut()[1] = 2;
        pileup.g_mut()[0] = 3;
        pileup.t_mut()[1] = 4;
        pileup.n_mut()[0] = 5;
        pileup.deletion_mut()[1] = 6;

        assert_eq!(pileup.a(), &[1, 0]);
        assert_eq!(pileup.c(), &[0, 2]);
        assert_eq!(pileup.g(), &[3, 0]);
        assert_eq!(pileup.t(), &[0, 4]);
        assert_eq!(pileup.n(), &[5, 0]);
        assert_eq!(pileup.deletion(), &[0, 6]);
    }

    #[test]
    fn reset_resizes_and_zeros() {
        let mut pileup = Pileup::<u32>::zeros(2);
        pileup[Observed::A][0] = 7;
        pileup[Observed::Deletion][1] = 3;

        pileup.reset(3);
        assert_eq!(pileup.len(), 3);
        assert_eq!(pileup.a(), &[0, 0, 0]);
        assert_eq!(pileup.deletion(), &[0, 0, 0]);

        pileup[Observed::C][2] = 5;
        pileup.reset(1);
        assert_eq!(pileup.len(), 1);
        assert_eq!(pileup.c(), &[0]);
    }

    #[test]
    fn iterates_sites() {
        let pileup = Pileup::<u32>::zeros(3);

        let mut sites = pileup.iter();
        assert_eq!(sites.len(), 3);

        let site = sites.next().unwrap();
        assert_eq!(site.offset(), 0);
        assert_eq!(site[Observed::A], 0);
        assert_eq!(*site.a(), 0);
        assert_eq!(sites.len(), 2);
        assert_eq!(
            sites.map(|site| site.offset()).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn iter_mutates_sites() {
        let mut pileup = Pileup::<u32>::zeros(3);

        {
            let mut sites = pileup.iter_mut();
            assert_eq!(sites.len(), 3);

            let mut site = sites.next().unwrap();
            assert_eq!(site.offset(), 0);
            assert_eq!(site[Observed::A], 0);
            *site.a_mut() = 100;
            site[Observed::C] = 2;
            *site.deletion_mut() = 10;
            assert_eq!(*site.a(), 100);
            assert_eq!(*site.c(), 2);
            assert_eq!(*site.deletion(), 10);
            assert_eq!(site.coverage(), 112);
            assert_eq!(site.matches(Reference::A), 100);
            assert_eq!(site.mismatches(Reference::A), 12);
            assert!(site.is_covered());
            assert_eq!(sites.len(), 2);

            for mut site in sites {
                site[Observed::A] = site.offset() as u32 + 1;
                *site.observed_mut(Observed::N) = site.offset() as u32 + 10;
            }
        }

        assert_eq!(pileup.a(), &[100, 2, 3]);
        assert_eq!(pileup.c(), &[2, 0, 0]);
        assert_eq!(pileup.n(), &[0, 11, 12]);
        assert_eq!(pileup.deletion(), &[10, 0, 0]);
    }

    #[test]
    fn mut_iterators_preserve_auto_traits() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SitesMut<'static, u32>>();
    }

    #[test]
    fn creates_sites_by_offset() -> Result<()> {
        let mut pileup = Pileup::<u32>::new(
            vec![1, 2],
            vec![3, 4],
            vec![5, 6],
            vec![7, 8],
            vec![9, 10],
            vec![11, 12],
        )?;

        let site = pileup.site(1);
        assert_eq!(site.offset(), 1);
        assert_eq!(*site.a(), 2);
        assert_eq!(*site.c(), 4);
        assert_eq!(*site.g(), 6);
        assert_eq!(*site.t(), 8);
        assert_eq!(*site.n(), 10);
        assert_eq!(*site.deletion(), 12);

        {
            let mut site = pileup.site_mut(0);
            *site.a_mut() = 21;
            *site.c_mut() = 22;
            *site.g_mut() = 23;
            *site.t_mut() = 24;
            *site.n_mut() = 25;
            *site.deletion_mut() = 26;
            assert_eq!(site[Observed::A], 21);
            *site.observed_mut(Observed::N) = 28;
        }

        assert_eq!(pileup.a(), &[21, 2]);
        assert_eq!(pileup.c(), &[22, 4]);
        assert_eq!(pileup.g(), &[23, 6]);
        assert_eq!(pileup.t(), &[24, 8]);
        assert_eq!(pileup.n(), &[28, 10]);
        assert_eq!(pileup.deletion(), &[26, 12]);
        Ok(())
    }

    #[test]
    fn site_calculates_counts_coverage_and_mismatches() -> Result<()> {
        let pileup = Pileup::<u32>::new(
            vec![2, 1, 1, 1, 1],
            vec![1, 2, 1, 1, 1],
            vec![1, 1, 2, 1, 1],
            vec![1, 1, 1, 2, 1],
            vec![1, 1, 1, 1, 2],
            vec![1, 1, 1, 1, 1],
        )?;

        let sites = pileup.iter().collect::<Vec<_>>();

        assert_eq!(pileup.iter().len(), 5);
        assert_eq!(sites[0].offset(), 0);
        assert_eq!(sites[0][Observed::A], 2);
        assert_eq!(*sites[0].a(), 2);
        assert_eq!(sites[0].coverage(), 7);
        assert_eq!(sites[0].matches(Reference::A), 2);
        assert_eq!(sites[0].mismatches(Reference::A), 5);
        assert!(sites[0].is_covered());
        let reference = [
            Reference::A,
            Reference::C,
            Reference::G,
            Reference::T,
            Reference::N,
        ];
        assert_eq!(
            sites
                .iter()
                .zip(reference)
                .map(|(site, reference)| site.mismatches(reference))
                .collect::<Vec<_>>(),
            vec![5, 5, 5, 5, 5]
        );
        Ok(())
    }

    #[test]
    fn site_coverage_and_mismatches_saturate() -> Result<()> {
        let pileup = Pileup::<u8>::new(vec![200], vec![100], vec![0], vec![0], vec![0], vec![0])?;
        let site = pileup.site(0);

        assert_eq!(site.coverage(), u8::MAX);
        assert_eq!(site.mismatches(Reference::A), 100);
        Ok(())
    }
}
