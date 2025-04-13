use biobit_core_rs::num::PrimInt;

use super::results::BatchHits;
use super::*;
use biobit_core_rs::loc::Interval;
use itertools::izip;
use itertools::Itertools;
use std::fmt::Debug;
use std::hash::Hash;

pub fn run_all<T>(builder: T)
where
    T: Builder<Target: ITree<Idx = i64, Data = &'static str>> + Clone,
{
    test_empty_tree(builder.clone());
    test_single_interval_tree(builder.clone());
    test_multi_interval_tree(builder.clone());
    test_deeply_nested_interval_tree(builder.clone());
    test_sparse_interval_tree(builder.clone());
}

fn normit<Idx: PrimInt>(input: impl Iterator<Item = (Idx, Idx)>) -> Vec<Interval<Idx>> {
    input
        .map(|(start, stop)| Interval::new(start, stop).unwrap())
        .collect()
}
fn normit_with_data<Idx: PrimInt, Data: Copy>(
    input: impl Iterator<Item = (Idx, Idx, Data)>,
) -> Vec<(Interval<Idx>, Data)> {
    input
        .map(|(start, stop, data)| (Interval::new(start, stop).unwrap(), data))
        .collect()
}

fn run_interval_test<Idx, Element, T>(
    builder: T,
    setup: &[(Idx, Idx, Element)],
    expected: &[(Idx, Idx, Vec<usize>)],
) where
    T: Builder<Target: ITree<Idx = Idx, Data = Element>>,
    Idx: PrimInt + Debug + Hash,
    Element: PartialEq + Eq + Hash + Ord + Debug + Copy,
{
    let setup = normit_with_data(setup.into_iter().cloned());
    let queries = normit(expected.iter().map(|(start, stop, _)| (*start, *stop)));
    let expected = expected
        .iter()
        .map(|(_, _, exp)| exp.iter().map(|i| setup[*i]).collect_vec())
        .collect_vec();

    let tree = builder.extend(setup.clone().into_iter()).build();

    // Batch mode
    let mut result = BatchHits::default();
    tree.batch_intersect_intervals(&queries, &mut result);

    assert_eq!(result.len(), expected.len());
    for (res, exp) in result.iter().zip(expected.iter()) {
        let intervals = res.0.iter().cloned().collect_vec();
        let data = res.1.iter().cloned().collect_vec();

        let res = izip!(intervals.into_iter(), data.into_iter().cloned())
            .unique()
            .sorted();
        let exp = exp.into_iter().unique().cloned().sorted();
        assert_eq!(res.collect_vec(), exp.collect_vec());
    }

    // Per-interval mode
    let mut results = Hits::default();
    for (query, exp) in izip!(queries.iter(), expected.iter()) {
        tree.intersect_interval(query, &mut results);
        let res = results
            .iter()
            .map(|x| (x.0.clone(), x.1.clone()))
            .unique()
            .sorted()
            .collect_vec();
        let exp = exp.into_iter().unique().cloned().sorted().collect_vec();
        assert_eq!(res, exp);
    }
}

fn test_empty_tree<T>(builder: T)
where
    T: Builder<Target: ITree<Idx = i64, Data = &'static str>>,
{
    run_interval_test(builder, &[], &[(0, 10, vec![]), (5, 15, vec![])]);
}

fn test_single_interval_tree<T>(builder: T)
where
    T: Builder<Target: ITree<Idx = i64, Data = &'static str>>,
{
    run_interval_test(
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
    T: Builder<Target: ITree<Idx = i64, Data = &'static str>>,
{
    run_interval_test(
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

fn test_deeply_nested_interval_tree<T>(builder: T)
where
    T: Builder<Target: ITree<Idx = i64, Data = &'static str>>,
{
    run_interval_test(
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

fn test_sparse_interval_tree<T>(builder: T)
where
    T: Builder<Target: ITree<Idx = i64, Data = &'static str>>,
{
    run_interval_test(
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
