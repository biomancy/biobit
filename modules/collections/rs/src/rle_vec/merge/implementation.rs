use std::cmp::Ordering;

use biobit_core_rs::num::PrimUInt;

use super::TryMerge;
use crate::rle_vec::{Identical, RleVec};

#[inline(always)]
fn append_run<V, L: PrimUInt, I: Identical<V>>(
    append_to: &mut RleVec<V, L, I>,
    current_value: &mut V,
    current_length: &mut L,
    value: V,
    length: L,
) {
    debug_assert!(length > L::zero());

    if append_to.identical(current_value, &value)
        && let Some(length) = current_length.checked_add(&length)
    {
        *current_length = length;
    } else {
        append_to.push(
            std::mem::replace(current_value, value),
            std::mem::replace(current_length, length),
        );
    }
}

pub(super) fn merge_impl<V, L, IOriginal, INew, M>(
    first: &RleVec<V, L, IOriginal>,
    second: &RleVec<V, L, IOriginal>,
    mut append_to: RleVec<V, L, INew>,
    mut merge: M,
) -> Result<RleVec<V, L, INew>, M::Error>
where
    L: PrimUInt,
    IOriginal: Identical<V>,
    INew: Identical<V>,
    M: TryMerge<V>,
{
    let mut first_iter = first.runs().map(|(value, length)| (value, *length));
    let mut second_iter = second.runs().map(|(value, length)| (value, *length));

    let Some((mut first_value, mut first_length)) = first_iter.next() else {
        let Some((value, length)) = second_iter.next() else {
            return Ok(append_to);
        };

        let mut current_value = merge.second(value)?;
        let mut current_length = length;
        for (value, length) in second_iter {
            let value = merge.second(value)?;
            append_run(
                &mut append_to,
                &mut current_value,
                &mut current_length,
                value,
                length,
            );
        }
        append_to.push(current_value, current_length);
        return Ok(append_to);
    };

    let Some((mut second_value, mut second_length)) = second_iter.next() else {
        let mut current_value = merge.first(first_value)?;
        let mut current_length = first_length;
        for (value, length) in first_iter {
            let value = merge.first(value)?;
            append_run(
                &mut append_to,
                &mut current_value,
                &mut current_length,
                value,
                length,
            );
        }
        append_to.push(current_value, current_length);
        return Ok(append_to);
    };

    // Seed the output so the hot paths can keep a concrete accumulator.
    let mut ordering = first_length.cmp(&second_length);
    let mut current_length = match ordering {
        Ordering::Greater => second_length,
        Ordering::Less | Ordering::Equal => first_length,
    };
    let mut current_value = merge.both(first_value, second_value)?;

    // Merge the shared prefix. Each iteration advances at least one input run.
    let (first_tail, second_tail) = loop {
        match ordering {
            Ordering::Less => {
                second_length = second_length - first_length;
                let Some(run) = first_iter.next() else {
                    break (None, Some((second_value, second_length)));
                };
                (first_value, first_length) = run;
            }
            Ordering::Greater => {
                first_length = first_length - second_length;
                let Some(run) = second_iter.next() else {
                    break (Some((first_value, first_length)), None);
                };
                (second_value, second_length) = run;
            }
            Ordering::Equal => match (first_iter.next(), second_iter.next()) {
                (Some(first_run), Some(second_run)) => {
                    (first_value, first_length) = first_run;
                    (second_value, second_length) = second_run;
                }
                (Some(first_run), None) => break (Some(first_run), None),
                (None, Some(second_run)) => break (None, Some(second_run)),
                (None, None) => break (None, None),
            },
        }

        ordering = first_length.cmp(&second_length);
        let length = match ordering {
            Ordering::Greater => second_length,
            Ordering::Less | Ordering::Equal => first_length,
        };
        let value = merge.both(first_value, second_value)?;
        append_run(
            &mut append_to,
            &mut current_value,
            &mut current_length,
            value,
            length,
        );
    };

    // At most one input has a tail after the shared prefix.
    if let Some((value, length)) = first_tail {
        let value = merge.first(value)?;
        append_run(
            &mut append_to,
            &mut current_value,
            &mut current_length,
            value,
            length,
        );
        for (value, length) in first_iter {
            let value = merge.first(value)?;
            append_run(
                &mut append_to,
                &mut current_value,
                &mut current_length,
                value,
                length,
            );
        }
    } else if let Some((value, length)) = second_tail {
        let value = merge.second(value)?;
        append_run(
            &mut append_to,
            &mut current_value,
            &mut current_length,
            value,
            length,
        );
        for (value, length) in second_iter {
            let value = merge.second(value)?;
            append_run(
                &mut append_to,
                &mut current_value,
                &mut current_length,
                value,
                length,
            );
        }
    }

    append_to.push(current_value, current_length);

    Ok(append_to)
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::*;
    use crate::rle_vec::merge;

    type RleVector = RleVec<u8, u8, fn(&u8, &u8) -> bool>;

    fn maximum(first: &u8, second: &u8) -> Result<u8, Infallible> {
        Ok(*first.max(second))
    }

    fn from_dense(values: Vec<u8>) -> RleVector {
        RleVector::builder(PartialEq::eq)
            .with_dense_values_inplace(values)
            .unwrap()
            .build()
    }

    fn assert_merged_eq(vec1: &RleVector, vec2: &RleVector, items: Vec<(u8, u8)>) {
        for (first, second) in [(vec1, vec2), (vec2, vec1)] {
            let merged = merge(first, second)
                .with_merge_fns(|x| Ok(*x), maximum)
                .with_identical(PartialEq::eq)
                .build()
                .unwrap()
                .run()
                .unwrap();
            assert_eq!(
                merged.runs().map(|(x, y)| (*x, *y)).collect::<Vec<_>>(),
                items
            );
        }
    }

    #[test]
    fn merges_two_empty_inputs() {
        let rle1 = RleVector::builder(PartialEq::eq).build();
        let rle2 = rle1.clone();

        assert_merged_eq(&rle1, &rle2, vec![]);
    }

    #[test]
    fn splits_runs_that_overflow_length_type() {
        let first = RleVector::builder(PartialEq::eq)
            .with_rle_values(vec![1, 2], vec![200, 100])
            .unwrap()
            .build();
        let second = RleVector::builder(PartialEq::eq).build();

        let merged = merge(&first, &second)
            .with_merge_fns(|_| Ok::<_, Infallible>(0), |_, _| Ok::<_, Infallible>(0))
            .with_identical(PartialEq::eq)
            .build()
            .unwrap()
            .run()
            .unwrap();

        assert_eq!(
            merged
                .runs()
                .map(|(value, length)| (*value, *length))
                .collect::<Vec<_>>(),
            vec![(0, 200), (0, 100)]
        );
    }

    #[test]
    fn does_not_observe_zero_length_runs() {
        let first = RleVector::builder(PartialEq::eq)
            .with_rle_values(vec![1, 99], vec![1, 0])
            .unwrap()
            .build();
        let second = from_dense(vec![2]);

        let merged = merge(&first, &second)
            .with_merge_fns(
                |_: &u8| Err("exclusive callback observed a zero-length run"),
                |first: &u8, second: &u8| Ok(*first.max(second)),
            )
            .with_identical(PartialEq::eq)
            .build()
            .unwrap()
            .run()
            .unwrap();

        assert_eq!(merged.into_iter().collect::<Vec<_>>(), vec![(2, 1)]);
    }

    #[test]
    fn merges_one_empty_input() {
        let rle1 = RleVector::builder(PartialEq::eq).build();
        let rle2 = from_dense(vec![1, 2, 2]);

        assert_merged_eq(&rle1, &rle2, vec![(1, 1), (2, 2)]);
    }

    #[test]
    fn merges_single_runs() {
        let rle1 = from_dense(vec![3, 3, 3]);
        let rle2 = from_dense(vec![1, 2, 4]);

        assert_merged_eq(&rle1, &rle2, vec![(3, 2), (4, 1)]);
    }

    #[test]
    fn merges_multiple_runs() {
        let rle1 = from_dense(vec![3, 4, 5, 6, 6, 6]);
        let rle2 = from_dense(vec![1, 2, 3, 4, 5, 6]);

        assert_merged_eq(&rle1, &rle2, vec![(3, 1), (4, 1), (5, 1), (6, 3)]);
    }

    #[test]
    fn merges_inputs_with_different_lengths() {
        let rle1 = from_dense(vec![1, 1, 2, 2, 3, 3, 4, 4, 5]);
        let rle2 = from_dense(vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6]);

        assert_merged_eq(
            &rle1,
            &rle2,
            vec![(1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (6, 1)],
        );
    }

    #[test]
    fn merges_single_and_multiple_runs() {
        let rle1 = from_dense(vec![3]);
        let rle2 = from_dense(vec![1, 2, 4]);

        assert_merged_eq(&rle1, &rle2, vec![(3, 1), (2, 1), (4, 1)]);
    }

    #[test]
    fn merges_identical_inputs() {
        let rle = from_dense(vec![1, 2, 2, 3]);
        assert_merged_eq(&rle, &rle, vec![(1, 1), (2, 2), (3, 1)]);
    }

    #[test]
    fn merges_different_lengths_with_overlapping_values() {
        let rle1 = from_dense(vec![1, 100]);
        let rle2 = from_dense(vec![3, 4, 5, 6, 7]);

        assert_merged_eq(&rle1, &rle2, vec![(3, 1), (100, 1), (5, 1), (6, 1), (7, 1)]);
    }

    #[test]
    fn distinguishes_input_tails() {
        let first = from_dense(vec![1, 2]);
        let second = from_dense(vec![3]);

        let first_tail = merge(&first, &second)
            .with_merge((
                |value: &u8| Ok::<_, Infallible>(*value + 10),
                |value: &u8| Ok::<_, Infallible>(*value + 20),
                |first: &u8, second: &u8| Ok::<_, Infallible>(*first + *second),
            ))
            .with_identical(PartialEq::eq)
            .build()
            .unwrap()
            .run()
            .unwrap();
        assert_eq!(
            first_tail.into_iter().collect::<Vec<_>>(),
            vec![(4, 1), (12, 1)]
        );

        let second_tail = merge(&second, &first)
            .with_merge((
                |value: &u8| Ok::<_, Infallible>(*value + 10),
                |value: &u8| Ok::<_, Infallible>(*value + 20),
                |first: &u8, second: &u8| Ok::<_, Infallible>(*first + *second),
            ))
            .with_identical(PartialEq::eq)
            .build()
            .unwrap()
            .run()
            .unwrap();
        assert_eq!(
            second_tail.into_iter().collect::<Vec<_>>(),
            vec![(4, 1), (22, 1)]
        );
    }

    #[test]
    fn propagates_both_errors() {
        let rle1 = from_dense(vec![1]);
        let rle2 = from_dense(vec![2]);

        let result = merge(&rle1, &rle2)
            .with_merge_fns(
                |value| Ok(*value),
                |_, _| Err("failed to merge overlapping values"),
            )
            .with_identical(PartialEq::eq)
            .build()
            .unwrap()
            .run();

        let error = match result {
            Ok(_) => panic!("fallible merge unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(error, "failed to merge overlapping values");
    }

    #[test]
    fn propagates_single_errors() {
        let rle1 = from_dense(vec![1]);
        let rle2 = from_dense(vec![2, 3]);

        let result = merge(&rle1, &rle2)
            .with_merge_fns(
                |_: &u8| Err("unmatched input tail"),
                |first: &u8, second: &u8| Ok(*first.max(second)),
            )
            .with_identical(PartialEq::eq)
            .build()
            .unwrap()
            .run();

        let error = match result {
            Ok(_) => panic!("fallible merge unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(error, "unmatched input tail");
    }
}
