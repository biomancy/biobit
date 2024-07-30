use std::iter::Zip;
use std::vec::IntoIter;

use derive_getters::Dissolve;
use eyre::Result;

use biobit_core_rs::num::PrimUInt;

use super::identical::Identical;

pub struct RleVecBuilder<V, L: PrimUInt, I: Identical<V>> {
    values: Option<Vec<V>>,
    lengths: Option<Vec<L>>,
    identical: I,
}

impl<V, L: PrimUInt, I: Identical<V>> RleVecBuilder<V, L, I> {
    pub fn new(identity: I) -> Self {
        Self {
            values: None,
            lengths: None,
            identical: identity,
        }
    }

    pub fn with_identical<NewI: Identical<V>>(
        self,
        identical: NewI,
    ) -> RleVecBuilder<V, L, NewI> {
        RleVecBuilder {
            values: self.values,
            lengths: self.lengths,
            identical,
        }
    }

    pub fn with_capacity(mut self, values: usize, lengths: usize) -> Self {
        self.values = Some(Vec::with_capacity(values));
        self.lengths = Some(Vec::with_capacity(lengths));
        self
    }

    pub fn with_buffers(mut self, mut values: Vec<V>, mut lengths: Vec<L>) -> Self {
        values.clear();
        lengths.clear();

        self.values = Some(values);
        self.lengths = Some(lengths);
        self
    }

    pub fn with_rle_values(mut self, values: Vec<V>, lengths: Vec<L>) -> Result<Self> {
        if values.len() != lengths.len() {
            return Err(eyre::eyre!(
                "Values and lengths must have the same length, got {} and {}",
                values.len(),
                lengths.len()
            ));
        }

        self.values = Some(values);
        self.lengths = Some(lengths);
        Ok(self)
    }

    pub fn with_dense_values(mut self, mut values: Vec<V>) -> Result<Self> {
        if values.len() == 0 {
            return Ok(self);
        } else if values.len() == 1 {
            self.values = Some(values);
            self.lengths = Some(vec![L::one()]);
            return Ok(self);
        }

        let mut lengths = self.lengths.take().unwrap_or_default();
        lengths.clear();

        let mut current = 0;
        let mut length: u64 = 1;

        let mut cursor = 1;
        while cursor < values.len() {
            if !self.identical.identical(&values[current], &values[cursor]) {
                lengths.push(L::from(length).ok_or_else(|| {
                    eyre::eyre!("Length {} can't fit in {:?}", length, L::max_value())
                })?);
                debug_assert_eq!(lengths.len(), current + 1);

                length = 0;
                current += 1;
                values.swap(current, cursor);
            }

            length += 1;
            cursor += 1;
        }
        debug_assert!(length > 0);

        lengths.push(
            L::from(length).ok_or_else(|| {
                eyre::eyre!("Length {} can't fit in {:?}", length, L::max_value())
            })?,
        );

        values.truncate(current + 1);
        debug_assert_eq!(values.len(), lengths.len());

        self.values = Some(values);
        self.lengths = Some(lengths);
        Ok(self)
    }

    pub fn build(self) -> RleVec<V, L, I> {
        let values = self.values.unwrap_or_default();
        let lengths = self.lengths.unwrap_or_default();
        let identical = self.identical;

        RleVec {
            values,
            lengths,
            identical,
        }
    }
}

#[derive(Debug, Clone, Default, Dissolve)]
pub struct RleVec<V, L: PrimUInt, I: Identical<V>> {
    values: Vec<V>,
    lengths: Vec<L>,
    identical: I,
}

impl<V, L: PrimUInt, I: Identical<V>> RleVec<V, L, I> {
    pub fn builder(identity: I) -> RleVecBuilder<V, L, I> {
        RleVecBuilder::new(identity)
    }

    pub fn rebuild(self) -> RleVecBuilder<V, L, I> {
        RleVecBuilder {
            values: Some(self.values),
            lengths: Some(self.lengths),
            identical: self.identical,
        }
    }

    pub fn identical(&self, first: &V, second: &V) -> bool {
        self.identical.identical(first, second)
    }

    pub fn is_empty(&self) -> bool {
        self.lengths.is_empty()
    }

    pub fn len(&self) -> usize {
        self.lengths.len()
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.lengths.clear();
    }

    pub fn push(&mut self, value: V, length: L) {
        self.values.push(value);
        self.lengths.push(length);
    }

    pub fn pop(&mut self) -> Option<(V, L)> {
        match (self.values.pop(), self.lengths.pop()) {
            (Some(value), Some(length)) => Some((value, length)),
            _ => None,
        }
    }

    pub fn extend<Iter: IntoIterator<Item = (V, L)>>(&mut self, iter: Iter) {
        for (value, length) in iter {
            self.push(value, length);
        }
    }

    pub fn runs(&self) -> impl Iterator<Item = (&V, &L)> {
        self.values.iter().zip(self.lengths.iter())
    }
}

impl<V, L: PrimUInt, I: Identical<V>> IntoIterator for RleVec<V, L, I> {
    type Item = (V, L);
    type IntoIter = Zip<IntoIter<V>, IntoIter<L>>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter().zip(self.lengths.into_iter())
    }
}

impl<V, L: PrimUInt, I: Identical<V>> Into<(Vec<V>, Vec<L>)> for RleVec<V, L, I> {
    fn into(self) -> (Vec<V>, Vec<L>) {
        (self.values, self.lengths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type RleVector = RleVec<u8, u8, fn(&u8, &u8) -> bool>;

    fn construct_from_dense(values: Vec<u8>) -> RleVector {
        RleVector::builder(PartialEq::eq)
            .with_dense_values(values)
            .unwrap()
            .build()
    }

    fn assert_rle_eq(vec: RleVector, items: Vec<(u8, u8)>) {
        assert_eq!(vec.runs().map(|(x, y)| (*x, *y)).collect::<Vec<_>>(), items);
    }

    #[test]
    fn test_rle_vec_with_dense_values() -> Result<()> {
        for (values, expected) in [
            (vec![], vec![]),
            (vec![1], vec![(1, 1)]),
            (vec![1, 1], vec![(1, 2)]),
            (vec![1, 1, 1], vec![(1, 3)]),
            (vec![1, 2], vec![(1, 1), (2, 1)]),
            (vec![1, 1, 2], vec![(1, 2), (2, 1)]),
            (vec![1, 1, 2, 2], vec![(1, 2), (2, 2)]),
            (vec![1, 1, 2, 2, 3], vec![(1, 2), (2, 2), (3, 1)]),
            (vec![1, 1, 1, 2, 2, 3], vec![(1, 3), (2, 2), (3, 1)]),
            (
                vec![1, 2, 3, 4, 5],
                vec![(1, 1), (2, 1), (3, 1), (4, 1), (5, 1)],
            ),
            (
                vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5],
                vec![(1, 2), (2, 2), (3, 2), (4, 2), (5, 2)],
            ),
            (
                vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6],
                vec![(1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (6, 1)],
            ),
        ] {
            assert_rle_eq(construct_from_dense(values), expected);
        }
        Ok(())
    }
}
