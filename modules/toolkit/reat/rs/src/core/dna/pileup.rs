use std::ops::{Index, IndexMut};

use biobit_core_rs::num::PrimUInt;

use crate::core::dna::Observed;

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Pileup<T: PrimUInt = u32> {
    pub a: Vec<T>,
    pub c: Vec<T>,
    pub g: Vec<T>,
    pub t: Vec<T>,
    pub n: Vec<T>,
    pub deletion: Vec<T>,
    pub insertion: Vec<T>,
}

impl<T: PrimUInt> Pileup<T> {
    pub fn new(
        a: Vec<T>,
        c: Vec<T>,
        g: Vec<T>,
        t: Vec<T>,
        n: Vec<T>,
        deletion: Vec<T>,
        insertion: Vec<T>,
    ) -> Self {
        let len = a.len();
        assert_eq!(c.len(), len);
        assert_eq!(g.len(), len);
        assert_eq!(t.len(), len);
        assert_eq!(n.len(), len);
        assert_eq!(deletion.len(), len);
        assert_eq!(insertion.len(), len);
        Self {
            a,
            c,
            g,
            t,
            n,
            deletion,
            insertion,
        }
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
            insertion: vec![T::default(); len],
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.a.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.a.is_empty()
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
            Observed::Insertion => &self.insertion,
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
            Observed::Insertion => &mut self.insertion,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeros() {
        let pileup = Pileup::<u32>::zeros(3);
        assert_eq!(pileup.len(), 3);
        assert_eq!(&pileup[Observed::A], &[0, 0, 0]);
        assert_eq!(&pileup[Observed::Deletion], &[0, 0, 0]);
        assert!(!pileup.is_empty());
    }

    #[test]
    fn new_validates_lengths() {
        let pileup = Pileup::<u32>::new(
            vec![1],
            vec![2],
            vec![3],
            vec![4],
            vec![5],
            vec![6],
            vec![7],
        );
        assert_eq!(pileup.len(), 1);
        assert_eq!(pileup[Observed::A][0], 1);
        assert_eq!(pileup[Observed::Insertion][0], 7);
    }

    #[test]
    #[should_panic]
    fn new_panics_on_mismatched_lengths() {
        Pileup::<u32>::new(
            vec![0],
            vec![0, 0],
            vec![0],
            vec![0],
            vec![0],
            vec![0],
            vec![0],
        );
    }

    #[test]
    fn index_mut() {
        let mut pileup = Pileup::<u32>::zeros(2);
        pileup[Observed::Insertion][1] = 3;
        assert_eq!(pileup.insertion, vec![0, 3]);
    }

    #[test]
    fn default_pileup_count_type_is_u32() {
        let pileup: Pileup = Pileup::zeros(1);
        let _: u32 = pileup.a[0];
    }
}
