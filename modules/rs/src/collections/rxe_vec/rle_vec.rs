use crate::collections::rxe_vec::Length;
use crate::collections::rxe_vec::traits::{Identical, Position};
use crate::collections::view::{View, ViewMut};

pub struct RleVecBuilder<V, I: Identical<V>, L: Length> {
    values: Option<Vec<V>>,
    lengths: Option<Vec<L>>,
    identical: Option<I>,
}

impl<V, I: Identical<V>, L: Length> RleVecBuilder<V, I, L> {
    #[inline]
    pub fn new() -> Self {
        Self {
            values: None,
            lengths: None,
            identical: None,
        }
    }

    #[inline]
    pub fn with_rle_data(mut self, values: Vec<V>, lengths: Vec<L>) -> Self {
        assert_eq!(values.len(), lengths.len(), "Values and lengths must have the same size");

        self.values = Some(values);
        self.lengths = Some(lengths);
        self
    }

    pub fn with_rpe_data<P: Position + Into<L>>(mut self, values: Vec<V>, positions: Vec<P>) -> Self {
        assert_eq!(values.len(), positions.len(), "Values and positions must have the same size");

        let mut previous = P::zero();
        let lengths = positions
            .into_iter()
            .map(|x| {
                let length = x - previous;
                previous = x;
                length.into()
            })
            .collect();
        self.values = Some(values);
        self.lengths = Some(lengths);
        self
    }

    #[inline]
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.values = Some(Vec::with_capacity(capacity));
        self.lengths = Some(Vec::with_capacity(capacity));
        self
    }

    #[inline]
    pub fn identity(mut self, identity: I) -> Self {
        self.identical = Some(identity);
        self
    }

    #[inline]
    pub fn build(self) -> RleVec<V, I, L> {
        let values = self.values.unwrap_or_default();
        let lengths = self.lengths.unwrap_or_default();
        let identical = self.identical.expect("Identical must be set");

        assert_eq!(values.len(), lengths.len(), "Values and lengths must have the same length");
        RleVec {
            values,
            lengths,
            identical,
        }
    }
}


pub struct RleVec<V, I: Identical<V>, L: Length = usize> {
    pub(crate) values: Vec<V>,
    pub(crate) lengths: Vec<L>,
    pub(crate) identical: I,
}

impl<V: PartialEq, L: Length> Default for RleVec<V, fn(&V, &V) -> bool, L> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            lengths: Vec::new(),
            identical: PartialEq::eq,
        }
    }
}

impl<V: PartialEq, L: Length> RleVec<V, fn(&V, &V) -> bool, L> {
    fn new() -> Self {
        Self::default()
    }
}

impl<V, I: Identical<V>, L: Length> RleVec<V, I, L> {
    pub fn builder() -> RleVecBuilder<V, I, L> {
        RleVecBuilder::new()
    }

    #[inline]
    pub fn dissolve(self) -> (Vec<V>, Vec<L>, I) {
        (self.values, self.lengths, self.identical)
    }

    #[inline]
    pub fn len(&self) -> usize { self.lengths.len() }

    #[inline]
    pub fn is_empty(&self) -> bool { self.lengths.is_empty() }

    #[inline]
    pub fn clear(&mut self) { (self.values.clear(), self.lengths.clear()); }
}

impl<V, I: Identical<V>, L: Length> View for RleVec<V, I, L> {
    type Output<'a> = RleVecView<'a, V, I, L> where Self: 'a;

    fn view(&self) -> Self::Output<'_> {
        RleVecView { rle: self }
    }
}

pub struct RleVecView<'a, V, I: Identical<V>, L: Length> {
    rle: &'a RleVec<V, I, L>,
}

impl<'a, V, I: Identical<V>, L: Length> RleVecView<'a, V, I, L> {
    pub fn flat(&self) {
        // self.rle.values
    }

    // pub fn runs(&self) -> RleVecRunsViewMut<T, M, Ind, &'a RleVec<T, M, Ind>> {
    //     RleVecRunsViewMut::new(self.rle)
    // }
}

impl<V, I: Identical<V>, L: Length> ViewMut for RleVec<V, I, L> {
    type Output<'a> = RleVecViewMut<'a, V, I, L> where Self: 'a;

    fn view_mut(&mut self) -> Self::Output<'_> {
        RleVecViewMut { rle: self }
    }
}

pub struct RleVecViewMut<'a, V, I: Identical<V>, L: Length> {
    rle: &'a mut RleVec<V, I, L>,
}

impl<'a, V, I: Identical<V>, L: Length> RleVecViewMut<'a, V, I, L> {
    pub fn flat(&self) {
        // self.rle.values
    }
}
