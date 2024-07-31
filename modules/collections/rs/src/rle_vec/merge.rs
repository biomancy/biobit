use std::marker::PhantomData;

use ::impl_tools::autoimpl;
use eyre::{eyre, Result};

use biobit_core_rs::num::PrimUInt;

use super::{Identical, RleVec};

#[autoimpl(for <M: trait + ?Sized> &mut M, Box<M>)]
pub trait Merge<T> {
    fn single(&mut self, val: &T) -> T;
    fn multiple(&mut self, vals: &[&T]) -> T;
}

pub struct MergeFn<T, Single, Multiple>
where
    Single: FnMut(&T) -> T,
    Multiple: FnMut(&[&T]) -> T,
{
    single_fn: Single,
    multiple_fn: Multiple,
    _phantom: PhantomData<T>,
}

impl<T, Single, Multiple> MergeFn<T, Single, Multiple>
where
    Single: FnMut(&T) -> T,
    Multiple: FnMut(&[&T]) -> T,
{
    pub fn new(single_fn: Single, multiple_fn: Multiple) -> Self {
        Self {
            single_fn,
            multiple_fn,
            _phantom: Default::default(),
        }
    }
}

impl<T, Single, Multiple> Merge<T> for MergeFn<T, Single, Multiple>
where
    Single: FnMut(&T) -> T,
    Multiple: FnMut(&[&T]) -> T,
{
    #[inline(always)]
    fn single(&mut self, val: &T) -> T {
        (self.single_fn)(val)
    }

    #[inline(always)]
    fn multiple(&mut self, vals: &[&T]) -> T {
        (self.multiple_fn)(vals)
    }
}

pub fn merge<'a, V, L, M, IOriginal, INew>(
    inputs: impl IntoIterator<Item = &'a RleVec<V, L, IOriginal>>,
) -> MergeSetup<'a, V, L, M, IOriginal, INew>
where
    L: PrimUInt,
    M: Merge<V>,
    IOriginal: Identical<V>,
    INew: Identical<V>,
{
    MergeSetup {
        inputs: inputs.into_iter().collect(),
        write_to: None,
        identical: None,
        merge: None,
    }
}

pub struct MergeSetup<'a, V, L: PrimUInt, M: Merge<V>, IOriginal: Identical<V>, INew: Identical<V>>
{
    inputs: Vec<&'a RleVec<V, L, IOriginal>>,
    write_to: Option<(Vec<V>, Vec<L>)>,
    identical: Option<INew>,
    merge: Option<M>,
}

impl<'a, V, L: PrimUInt, M: Merge<V>, IOriginal: Identical<V>, INew: Identical<V>>
    MergeSetup<'a, V, L, M, IOriginal, INew>
{
    pub fn save_to(mut self, buffer: impl Into<(Vec<V>, Vec<L>)>) -> Self {
        self.write_to = Some(buffer.into());
        self
    }

    pub fn with_identical(mut self, identical: INew) -> Self {
        self.identical = Some(identical);
        self
    }

    pub fn with_merge(mut self, merge: M) -> Self {
        self.merge = Some(merge);
        self
    }

    pub fn run(mut self) -> Result<RleVec<V, L, INew>> {
        let merge_fn = self
            .merge
            .take()
            .ok_or_else(|| eyre!("Merge function is unspecified in rle_vec::merge."))?;
        let identical = self
            .identical
            .take()
            .ok_or_else(|| eyre!("Identical rule is unspecified in rle_vec::merge."))?;

        let rle = self.write_to.take().unwrap_or_default();
        let rle = RleVec::builder(identical)
            .with_buffers(rle.0, rle.1)
            .build();

        merge_impl(&self.inputs, rle, merge_fn)
    }
}

fn merge_impl<V, L, IOriginal, INew>(
    inputs: &[&RleVec<V, L, IOriginal>],
    mut append_to: RleVec<V, L, INew>,
    mut merge: impl Merge<V>,
) -> Result<RleVec<V, L, INew>>
where
    L: PrimUInt,
    IOriginal: Identical<V>,
    INew: Identical<V>,
{
    // Iterators + cached current values
    let mut iterators = Vec::with_capacity(inputs.len());
    let mut iter_ends = Vec::with_capacity(inputs.len());
    let mut iter_vals = Vec::with_capacity(inputs.len());

    for rle in inputs {
        let mut rle = rle.runs();
        match rle.next() {
            None => continue,
            Some((val, length)) => {
                iter_ends.push(length.to_u64().ok_or_else(|| {
                    eyre!("Length {:?} can't fit in {:?}", length, L::max_value())
                })?);
                iter_vals.push(val);
                iterators.push(rle);
            }
        }
    }

    if iterators.is_empty() {
        return Ok(append_to);
    }

    // Init for the first iteration
    let mut current_end: u64 = 0;
    let mut current_length = L::zero();
    let mut current_value = merge.multiple(&iter_vals);

    while iterators.len() > 1 {
        debug_assert_eq!(iter_ends.len(), iterators.len());
        debug_assert_eq!(iter_vals.len(), iterators.len());

        // Next end is a min among active run ends
        let new_end = unsafe { iter_ends.iter().min().unwrap_unchecked() };

        // Next value is a transform of present values
        let new_value = merge.multiple(&iter_vals);

        let length = L::from(new_end - current_end)
            .ok_or_else(|| eyre!("Length can't fit in {:?}", L::max_value()))?;
        debug_assert!(length > L::zero());

        // Save only if it differs from the current value
        if append_to.identical(&current_value, &new_value) {
            current_length = current_length.checked_add(&length).unwrap();
        } else {
            append_to.push(current_value, current_length);

            current_value = new_value;
            current_length = length;
        }

        current_end = *new_end;

        // Push iterators forward and drop finished iterators
        let mut ind = 0;
        iterators.retain_mut(|x| {
            // Interval is not finished - nothing to do
            if iter_ends[ind] > current_end {
                ind += 1;
                return true;
            }

            // Interval is finished - try to get the next value
            debug_assert!(iter_ends[ind] == current_end);
            match x.next() {
                Some((val, length)) => {
                    iter_vals[ind] = val;
                    iter_ends[ind] += length.to_u64().unwrap();
                    ind += 1;
                    true
                }
                None => {
                    // Iterator is completely consumed - drop associated values and don't increment the index
                    iter_vals.remove(ind);
                    iter_ends.remove(ind);
                    false
                }
            }
        });
    }

    // Consume the last iterator
    if let Some(iter) = iterators.pop() {
        // Last cached value
        let new_end = iter_ends.pop().unwrap();
        let new_value = merge.single(iter_vals.pop().unwrap());

        let length = L::from(new_end - current_end)
            .ok_or_else(|| eyre!("Length can't fit in {:?}", L::max_value()))?;

        if append_to.identical(&current_value, &new_value) {
            current_length = current_length.checked_add(&length).unwrap();
        } else {
            append_to.push(current_value, current_length);

            current_value = new_value;
            current_length = length;
        }

        // Consume the rest of the iterator
        for (val, length) in iter {
            let val = merge.single(val);

            if append_to.identical(&current_value, &val) {
                current_length = current_length.checked_add(&length).unwrap();
            } else {
                append_to.push(current_value, current_length);

                current_value = val;
                current_length = *length;
            }
        }
    }

    assert!(current_length > L::zero());
    append_to.push(current_value, current_length);

    debug_assert!(iterators.is_empty());
    debug_assert!(iter_vals.is_empty());
    debug_assert!(iter_ends.is_empty());

    Ok(append_to)
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use super::*;

    type RleVector = RleVec<u8, u8, fn(&u8, &u8) -> bool>;

    fn maximum(values: &[&u8]) -> u8 {
        *values.iter().cloned().max().unwrap()
    }

    fn construct_from_dense(values: Vec<u8>) -> RleVector {
        RleVector::builder(PartialEq::eq)
            .with_dense_values_inplace(values)
            .unwrap()
            .build()
    }

    fn test_merge<'a>(rles: impl IntoIterator<Item = &'a RleVector>) -> Result<RleVector> {
        merge(rles)
            .with_merge(MergeFn::new(|x| *x, maximum))
            .with_identical(PartialEq::eq as fn(&u8, &u8) -> bool)
            .run()
    }

    fn assert_rle_eq(vec: RleVector, items: Vec<(u8, u8)>) {
        assert_eq!(vec.runs().map(|(x, y)| (*x, *y)).collect::<Vec<_>>(), items);
    }

    #[test]
    fn test_rle_vec_merge_single_empty() -> Result<()> {
        let vec = RleVector::builder(PartialEq::eq).build();
        let merged = test_merge(&[vec])?;

        debug_assert_eq!(merged.runs().count(), 0);
        debug_assert!(merged.is_empty());

        Ok(())
    }

    #[test]
    fn test_rle_vec_merge_single() -> Result<()> {
        let values = vec![1, 2, 3, 4, 5];
        let lengths = vec![1, 2, 3, 1, 1];

        let vec = RleVector::builder(|a, b| a == b)
            .with_rle_values(values.clone(), lengths.clone())?
            .build();

        let merged = test_merge(&[vec])?;

        assert_eq!(
            merged.runs().map(|(x, y)| (*x, *y)).collect::<Vec<_>>(),
            zip(values, lengths).collect::<Vec<_>>()
        );

        Ok(())
    }

    #[test]
    fn test_rle_vec_merge_multiple_empty() -> Result<()> {
        let vec1 = RleVector::builder(PartialEq::eq).build();
        let vec2 = RleVector::builder(PartialEq::eq).build();

        let merged = test_merge(&[vec1, vec2])?;

        assert!(merged.is_empty());
        Ok(())
    }

    #[test]
    fn test_rle_vec_merge_multiple_1() -> Result<()> {
        let rle1 = construct_from_dense(vec![1, 2, 3, 4, 5, 5, 4]);
        let rle2 = construct_from_dense(vec![]);
        let rle3 = construct_from_dense(vec![5, 5, 5, 5, 5, 1, 1, 1, 1]);
        let rle4 = construct_from_dense(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 1]);
        let rle5 = RleVector::builder(PartialEq::eq)
            .with_rle_values(vec![0], vec![100])?
            .build();

        let merged = test_merge(&[rle1, rle2, rle3, rle4, rle5])?;

        assert_rle_eq(
            merged,
            vec![(5, 6), (4, 1), (1, 2), (0, 2), (10, 1), (1, 1), (0, 87)],
        );
        Ok(())
    }

    #[test]
    fn test_rle_merge_multiple_2() -> Result<()> {
        let rle1 = construct_from_dense(vec![100, 100, 100]);
        let rle2 = construct_from_dense(vec![100, 100, 100]);
        let rle3 = construct_from_dense(vec![0, 0, 0, 100]);
        let rle4 = construct_from_dense(vec![0, 0, 0, 100]);

        let merged = test_merge(&[rle1, rle2, rle3, rle4])?;

        assert_rle_eq(merged, vec![(100, 4)]);

        Ok(())
    }

    #[test]
    fn test_rle_merge_multiple_single_element() -> Result<()> {
        let rle1 = construct_from_dense(vec![1]);
        let rle2 = construct_from_dense(vec![2]);

        let merged = test_merge(&[rle1, rle2])?;

        assert_rle_eq(merged, vec![(2, 1)]);
        Ok(())
    }

    #[test]
    fn test_rle_merge_multiple_single_rle() -> Result<()> {
        let rle = construct_from_dense(vec![123]);
        let merged = test_merge(&[rle])?;

        assert_rle_eq(merged, vec![(123, 1)]);
        Ok(())
    }
}
