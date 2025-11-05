use std::marker::PhantomData;

use ::impl_tools::autoimpl;
use eyre::{Result, eyre};

use biobit_core_rs::num::PrimUInt;

use super::{Identical, RleVec};

#[autoimpl(for <M: trait + ?Sized> &mut M, Box<M>)]
pub trait Merge2<T> {
    fn single(&mut self, val: &T) -> T;
    fn two(&mut self, first: &T, two: &T) -> T;
}

pub struct Merge2Fn<T, Single, Two>
where
    Single: FnMut(&T) -> T,
    Two: FnMut(&T, &T) -> T,
{
    single_fn: Single,
    two_fn: Two,
    _phantom: PhantomData<T>,
}

impl<T, Single, Two> Merge2Fn<T, Single, Two>
where
    Single: FnMut(&T) -> T,
    Two: FnMut(&T, &T) -> T,
{
    pub fn new(single_fn: Single, two_fn: Two) -> Self {
        Self {
            single_fn,
            two_fn,
            _phantom: Default::default(),
        }
    }
}

impl<T, Single, Two> Merge2<T> for Merge2Fn<T, Single, Two>
where
    Single: FnMut(&T) -> T,
    Two: FnMut(&T, &T) -> T,
{
    #[inline(always)]
    fn single(&mut self, val: &T) -> T {
        (self.single_fn)(val)
    }

    #[inline(always)]
    fn two(&mut self, first: &T, second: &T) -> T {
        (self.two_fn)(first, second)
    }
}

pub fn merge2<'a, V, L, M, IOriginal, INew>(
    first: &'a RleVec<V, L, IOriginal>,
    second: &'a RleVec<V, L, IOriginal>,
) -> Merge2Setup<'a, V, L, M, IOriginal, INew>
where
    L: PrimUInt,
    M: Merge2<V>,
    IOriginal: Identical<V>,
    INew: Identical<V>,
{
    Merge2Setup {
        first,
        second,
        write_to: None,
        identical: None,
        merge: None,
    }
}

pub struct Merge2Setup<
    'a,
    V,
    L: PrimUInt,
    M: Merge2<V>,
    IOriginal: Identical<V>,
    INew: Identical<V>,
> {
    first: &'a RleVec<V, L, IOriginal>,
    second: &'a RleVec<V, L, IOriginal>,
    write_to: Option<(Vec<V>, Vec<L>)>,
    identical: Option<INew>,
    merge: Option<M>,
}

impl<V, L: PrimUInt, M: Merge2<V>, IOriginal: Identical<V>, INew: Identical<V>>
    Merge2Setup<'_, V, L, M, IOriginal, INew>
{
    pub fn save_to(mut self, buffer: impl Into<(Vec<V>, Vec<L>)>) -> Self {
        let mut buffer = buffer.into();
        buffer.0.clear();
        buffer.1.clear();

        self.write_to = Some(buffer);
        self
    }

    pub fn with_identical(mut self, identical: INew) -> Self {
        self.identical = Some(identical);
        self
    }

    pub fn with_merge2(mut self, merge: M) -> Self {
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

        merge2_impl(self.first, self.second, rle, merge_fn)
    }
}

fn merge2_impl<V, L, IOriginal, INew>(
    first: &RleVec<V, L, IOriginal>,
    second: &RleVec<V, L, IOriginal>,
    mut append_to: RleVec<V, L, INew>,
    mut merge: impl Merge2<V>,
) -> Result<RleVec<V, L, INew>>
where
    L: PrimUInt,
    IOriginal: Identical<V>,
    INew: Identical<V>,
{
    // Iterators + cached current values
    let mut first = first.runs();
    let (mut first_val, first_end) = match first.next() {
        None => {
            append_to.extend(second.runs().map(|(v, l)| (merge.single(v), *l)));
            return Ok(append_to);
        }
        Some(x) => x,
    };
    let mut first_end = first_end.to_u64().unwrap();

    let mut second = second.runs();
    let (mut second_val, second_end) = match second.next() {
        None => {
            append_to.push(merge.single(first_val), L::from(first_end).unwrap());
            append_to.extend(first.map(|(v, l)| (merge.single(v), *l)));
            return Ok(append_to);
        }
        Some(x) => x,
    };
    let mut second_end = second_end.to_u64().unwrap();

    let mut current_end: u64 = 0;
    let mut current_length = L::zero();
    let mut current_value = merge.two(first_val, second_val);

    let mut first_iter_running = true;
    let mut second_iter_running = true;

    while first_iter_running && second_iter_running {
        // Next end is a min among first and second run ends
        let new_end = first_end.min(second_end);

        // Next value is a transform of current values
        let new_value = merge.two(first_val, second_val);

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

        current_end = new_end;

        // Push iterators forward
        if first_end == new_end {
            match first.next() {
                None => first_iter_running = false,
                Some((v, l)) => {
                    first_val = v;
                    first_end += l.to_u64().unwrap();
                }
            }
        }
        if second_end == new_end {
            match second.next() {
                None => second_iter_running = false,
                Some((v, l)) => {
                    second_val = v;
                    second_end += l.to_u64().unwrap();
                }
            }
        }
    }

    let unconsumed = match (first_iter_running, second_iter_running) {
        (true, true) => unreachable!("Both iterators should be finished"),
        (false, false) => {
            append_to.push(current_value, current_length);
            return Ok(append_to);
        }
        (true, false) => (first, first_val, first_end),
        (false, true) => (second, second_val, second_end),
    };
    let (iter, val, end) = unconsumed;

    // Consume cached iteration
    debug_assert!(end > current_end);

    let new_value = merge.single(val);
    let length = L::from(end - current_end)
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
            current_length = current_length.checked_add(length).unwrap();
        } else {
            append_to.push(current_value, current_length);

            current_value = val;
            current_length = *length;
        }
    }

    append_to.push(current_value, current_length);

    Ok(append_to)
}

#[cfg(test)]
mod tests {
    use super::*;

    type RleVector = RleVec<u8, u8, fn(&u8, &u8) -> bool>;

    fn maximum(val1: &u8, val2: &u8) -> u8 {
        *val1.max(val2)
    }

    fn construct_from_dense(values: Vec<u8>) -> RleVector {
        RleVector::builder(PartialEq::eq)
            .with_dense_values_inplace(values)
            .unwrap()
            .build()
    }

    fn assert_merged_eq(vec1: &RleVector, vec2: &RleVector, items: Vec<(u8, u8)>) {
        for (first, second) in [(vec1, vec2), (vec2, vec1)] {
            let merged = merge2(first, second)
                .with_merge2(Merge2Fn::new(|x| *x, maximum))
                .with_identical(PartialEq::eq)
                .run()
                .unwrap();
            assert_eq!(
                merged.runs().map(|(x, y)| (*x, *y)).collect::<Vec<_>>(),
                items
            );
        }
    }

    #[test]
    fn test_rle_vec_merge2_both_empty() {
        let rle1 = RleVector::builder(PartialEq::eq).build();
        let rle2 = rle1.clone();

        assert_merged_eq(&rle1, &rle2, vec![]);
    }

    #[test]
    fn test_rle_vec_merge2_single_empty() {
        let rle1 = RleVector::builder(PartialEq::eq).build();
        let rle2 = construct_from_dense(vec![1, 2, 2]);

        assert_merged_eq(&rle1, &rle2, vec![(1, 1), (2, 2)]);
    }

    #[test]
    fn test_rle_vec_merge2_single() {
        let rle1 = construct_from_dense(vec![3, 3, 3]);
        let rle2 = construct_from_dense(vec![1, 2, 4]);

        assert_merged_eq(&rle1, &rle2, vec![(3, 2), (4, 1)]);
    }

    #[test]
    fn test_rle_vec_merge2_multiple() {
        let rle1 = construct_from_dense(vec![3, 4, 5, 6, 6, 6]);
        let rle2 = construct_from_dense(vec![1, 2, 3, 4, 5, 6]);

        assert_merged_eq(&rle1, &rle2, vec![(3, 1), (4, 1), (5, 1), (6, 3)]);
    }

    #[test]
    fn test_rle_vec_merge2_multiple_different_len() {
        let rle1 = construct_from_dense(vec![1, 1, 2, 2, 3, 3, 4, 4, 5]);
        let rle2 = construct_from_dense(vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6]);

        assert_merged_eq(
            &rle1,
            &rle2,
            vec![(1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (6, 1)],
        );
    }

    #[test]
    fn test_rle_vec_merge2_single_and_multi_element() {
        let rle1 = construct_from_dense(vec![3]);
        let rle2 = construct_from_dense(vec![1, 2, 4]);

        assert_merged_eq(&rle1, &rle2, vec![(3, 1), (2, 1), (4, 1)]);
    }

    #[test]
    fn test_rle_vec_merge2_identical_instances() {
        let rle = construct_from_dense(vec![1, 2, 2, 3]);
        assert_merged_eq(&rle, &rle, vec![(1, 1), (2, 2), (3, 1)]);
    }

    #[test]
    fn test_rle_vec_merge2_different_lengths_overlapping_values() {
        let rle1 = construct_from_dense(vec![1, 100]);
        let rle2 = construct_from_dense(vec![3, 4, 5, 6, 7]);

        assert_merged_eq(&rle1, &rle2, vec![(3, 1), (100, 1), (5, 1), (6, 1), (7, 1)]);
    }
}
