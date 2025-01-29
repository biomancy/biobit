use crate::interval_tree::overlap;
use biobit_core_rs::{
    loc::{Interval, IntervalOp},
    num::PrimInt,
};

// Builder shouldn't include methods like 'append', which require elements to be Hash + Eq and
// merges overlapping intervals on the flight. This is better done in a separate step/module,
// because it introduces unnecessary complexity to the Builder trait and an extra constraint.
// Why not 2 builder traits? One for simple trees and one for merge-on-the-fly trees? Because
// in the nutshell the requirement to merge overlapping intervals is a separate concern from
// building the tree itself. It's better to keep them separate and have this merge-on-the-fly logic
// in a separate struct/module. Besides, it might be useful elsewhere, not only in the context of
// interval trees.
pub trait Builder {
    type Target: ITree;

    fn addi(
        self,
        interval: impl IntervalOp<Idx = <Self::Target as ITree>::Idx>,
        element: <Self::Target as ITree>::Element,
    ) -> Self;
    fn add(
        self,
        data: impl Iterator<
            Item = (
                <Self::Target as ITree>::Element,
                impl IntervalOp<Idx = <Self::Target as ITree>::Idx>,
            ),
        >,
    ) -> Self;
    fn build(self) -> Self::Target;
}

pub trait Record<'borrow, 'iter> {
    type Idx: PrimInt + 'iter;
    type Value: 'borrow;
    fn interval(&self) -> &'iter Interval<Self::Idx>;
    fn value(&self) -> &'borrow Self::Value;
}

pub trait ITree {
    type Idx: PrimInt;
    type Element: Clone;

    // Think about:
    // * How to make this work with gapped intervals?
    // * How to leverage the information about sorted queries?
    // * How to marry this with the LendingIterator functionality?
    // * How to make a difference between "overlap" and strict "overlap" where
    //   only(!) overlapping parts are returned and not the whole interval?
    // * How should it look in Python to allow long-term goal of a Pytorch-like experience?
    // * How to make this work with a tree that can be mutated?
    fn overlap_single_element<'a>(
        &self,
        intervals: &[Interval<Self::Idx>],
        buffer: &'a mut overlap::Elements<Self::Idx, Self::Element>,
    ) -> &'a mut overlap::Elements<Self::Idx, Self::Element>;
}

// pub trait LendingITree: ITree {
//     type Iter: for<'borrow, 'iter> ForLt<
//         Of<'borrow>: LendingIterator<
//             Item: ForLt<Of<'iter>: Record<'borrow, 'iter, Idx = Self::Idx, Value = Self::Element>>,
//         >,
//     >;
//
//     fn overlap<'borrow>(
//         &'borrow self,
//         intervals: &impl LendingIterator<Item: AsSegment<Idx = Self::Idx>>,
//     ) -> <Self::Iter as ForLt>::Of<'borrow>;
// }

impl<'borrow, 'iter, Idx: PrimInt, Val> Record<'borrow, 'iter>
    for (&'iter Interval<Idx>, &'borrow Val)
{
    type Idx = Idx;
    type Value = Val;

    fn interval(&self) -> &'iter Interval<Self::Idx> {
        self.0
    }

    fn value(&self) -> &'borrow Self::Value {
        self.1
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use itertools::Itertools;
    use std::fmt::Debug;

    fn run_test<Idx, Element, T>(
        mut builder: T,
        setup: &[(Idx, Idx, Element)],
        queries: &[(Idx, Idx, Vec<usize>)],
    ) where
        T: Builder<Target: ITree<Idx = Idx, Element = Element>>,
        Idx: PrimInt + Debug,
        Element: PartialEq + Debug + Copy,
    {
        // Construct the tree
        for (start, stop, element) in setup {
            builder = builder.addi(&Interval::new(*start, *stop).unwrap(), *element);
        }
        let tree = builder.build();

        // Run the queries
        let mut result = overlap::Elements::default();
        let mut expected: Vec<(Vec<Interval<Idx>>, Vec<Element>)> =
            Vec::with_capacity(queries.len());
        for (start, end, indices) in queries {
            tree.overlap_single_element(&[Interval::new(*start, *end).unwrap()], &mut result);
            expected.push(
                indices
                    .iter()
                    .map(|i| {
                        (
                            Interval::new(setup[*i].0, setup[*i].1).unwrap(),
                            &setup[*i].2,
                        )
                    })
                    .unzip(),
            );
        }
        assert_eq!(result.len(), expected.len());

        let observed = result.iter().collect_vec();
        assert_eq!(observed.len(), expected.len());

        for ind in 0..observed.len() {
            let (obs, exp) = (&observed[ind], &expected[ind]);
            assert_eq!(obs.0, exp.0, "Observed: {:?}\nExpected: {:?}", obs, exp);
            assert_eq!(obs.1.len(), exp.1.len());
            for (obs, exp) in obs.1.iter().zip(exp.1.iter()) {
                assert_eq!(*obs, *exp);
            }
        }
    }

    pub fn test_interval_tree<T>(builder: T)
    where
        T: Builder<Target: ITree<Idx = i64, Element = &'static str>> + Clone,
    {
        test_empty_tree(builder.clone());
        test_single_interval_tree(builder.clone());
        test_multi_interval_tree(builder.clone());
        test_deeply_nested_interval_tree(builder.clone());
        test_sparse_interval_tree(builder.clone());
    }

    pub fn test_empty_tree<T>(builder: T)
    where
        T: Builder<Target: ITree<Idx = i64, Element = &'static str>>,
    {
        run_test(builder, &[], &[(0, 10, vec![]), (5, 15, vec![])]);
    }

    pub fn test_single_interval_tree<T>(builder: T)
    where
        T: Builder<Target: ITree<Idx = i64, Element = &'static str>>,
    {
        run_test(
            builder,
            &[(10, 20, "a")],
            &[
                // Off-range queries
                (0, 5, vec![]),
                (21, 25, vec![]),
                // Touching query
                (0, 10, vec![]),
                (20, 30, vec![]),
                // Intersecting queries
                (5, 15, vec![0]),
                (15, 25, vec![0]),
                (5, 25, vec![0]),
                // Enveloping queries
                (10, 20, vec![0]),
                (9, 21, vec![0]),
            ],
        );
    }

    pub fn test_multi_interval_tree<T>(builder: T)
    where
        T: Builder<Target: ITree<Idx = i64, Element = &'static str>>,
    {
        run_test(
            builder,
            &[(1, 10, "a"), (5, 15, "b"), (10, 20, "c")],
            &[
                // Touching the first interval
                (0, 1, vec![]),
                // Left-intersect the first interval
                (0, 2, vec![0]),
                // Multi-intersect all intervals
                (5, 15, vec![0, 1, 2]),
                // Multi-intersect the last two intervals
                (14, 25, vec![1, 2]),
                // Right-intersect the last interval
                (19, 50, vec![2]),
                // Touching the last interval
                (20, 50, vec![]),
                // Enveloping all intervals
                (0, 21, vec![0, 1, 2]),
                (1, 20, vec![0, 1, 2]),
            ],
        );
    }

    pub fn test_deeply_nested_interval_tree<T>(builder: T)
    where
        T: Builder<Target: ITree<Idx = i64, Element = &'static str>>,
    {
        run_test(
            builder,
            &[
                (-100, 150, "a"),
                (0, 100, "b"),
                (10, 90, "c"),
                (20, 80, "d"),
                (30, 70, "e"),
                (40, 60, "f"),
                (45, 50, "g"),
                (47, 48, "h"),
                (48, 49, "i"),
            ],
            &[
                // Touching the first interval
                (-101, -100, vec![]),
                // Left-intersect the first interval
                (-101, -99, vec![0]),
                // Envelope all intervals
                (-100, 150, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]),
                (-101, 151, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]),
                // Intersect all but one interval
                (48, 49, vec![0, 1, 2, 3, 4, 5, 6, 8]),
                // Right-intersect first intervals
                (80, 100, vec![0, 1, 2]),
            ],
        );
    }

    pub fn test_sparse_interval_tree<T>(builder: T)
    where
        T: Builder<Target: ITree<Idx = i64, Element = &'static str>>,
    {
        run_test(
            builder,
            &[
                (-100, 100, "0"),
                (-10, 0, "a"),
                (10, 20, "b"),
                (30, 40, "c"),
                (50, 60, "d"),
                (200, 250, "e"),
                (225, 250, "f"),
                (300, 350, "g"),
                (400, 450, "h"),
            ],
            &[
                // Touching the first interval
                (-101, -100, vec![]),
                // Left-intersect the first interval
                (-100, -99, vec![0]),
                // Envelope all intervals
                (-100, 460, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]),
                (-101, 461, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]),
                // Envelope left intervals
                (-100, 60, vec![0, 1, 2, 3, 4]),
                // Envelope right intervals
                (60, 460, vec![0, 5, 6, 7, 8]),
                (100, 450, vec![5, 6, 7, 8]),
                // Intersect some intervals
                (-10, -8, vec![0, 1]),
                (-10, 30, vec![0, 1, 2]),
                (30, 70, vec![0, 3, 4]),
                (60, 230, vec![0, 5, 6]),
                // Intersect all intervals
                (-5, 410, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]),
                // Touch the last interval
                (450, 451, vec![]),
                // Right-intersect the last interval
                (449, 500, vec![8]),
            ],
        )
    }
}
