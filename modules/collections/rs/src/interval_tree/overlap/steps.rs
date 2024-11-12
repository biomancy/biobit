use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::num::PrimInt;
use derive_getters::Dissolve;
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;

#[derive(Clone, PartialEq, Eq, Debug, Dissolve)]
pub struct Steps<Idx: PrimInt, T: Eq + Hash> {
    cache: BTreeSet<Idx>,
    hitlen: Vec<usize>,
    // Boundaries & annotation for stepped hits
    boundaries: Vec<Idx>,
    annotation: Vec<HashSet<T>>,
}

impl<Idx: PrimInt, T: Eq + Hash + Clone> Steps<Idx, T> {
    pub fn with_capacity(hits: usize, boundaries: usize, annotation: usize) -> Self {
        Self {
            cache: BTreeSet::new(),
            hitlen: Vec::with_capacity(hits),
            boundaries: Vec::with_capacity(boundaries),
            annotation: Vec::with_capacity(annotation),
        }
    }

    pub fn empty() -> Self {
        Self::with_capacity(0, 0, 0)
    }

    pub fn build<'a>(
        &'a mut self,
        data: impl Iterator<Item = (&'a Interval<Idx>, (&'a [Interval<Idx>], &'a [T]))>,
    ) {
        // boundaries are of length N + 1
        // annotation is of length N
        // N is recorded in the hitlen for each query
        self.clear();
        let mut total = 0;
        for (query, (hits, annotations)) in data {
            self.cache.clear();
            self.cache.insert(query.start());
            self.cache.insert(query.end());
            for it in hits.iter() {
                if it.start() > query.start() {
                    self.cache.insert(it.start());
                }
                if it.end() < query.end() {
                    self.cache.insert(it.end());
                }
            }

            self.boundaries.extend(self.cache.iter());
            self.hitlen.push(self.cache.len() - 1);

            // Allocate enough space for all the annotations if needed
            if self.annotation.len() < total + self.cache.len() - 1 {
                self.annotation
                    .resize_with(total + self.cache.len() - 1, || HashSet::new());
            }

            // Populate stepped annotation for the current query
            let boundaries = &self.boundaries[total + self.hitlen.len() - 1..];
            let steps = &mut self.annotation[total..total + self.cache.len() - 1];
            debug_assert!(boundaries.len() == steps.len() + 1);
            for (it, anno) in hits.iter().zip(annotations) {
                let st = if it.start() <= query.start() {
                    0
                } else {
                    boundaries.binary_search(&it.start()).unwrap()
                };
                let en = if it.end() >= query.end() {
                    steps.len()
                } else {
                    boundaries.binary_search(&it.end()).unwrap()
                };

                for step in steps[st..en].iter_mut() {
                    step.insert(anno.clone());
                }
            }

            total += self.cache.len() - 1;
        }

        debug_assert!(total == self.boundaries.len() - self.hitlen.len());
        debug_assert!(total == self.hitlen.iter().sum());
    }

    pub fn iter(&self) -> impl Iterator<Item = impl Iterator<Item = (Idx, Idx, &HashSet<T>)>> {
        self.hitlen
            .iter()
            .scan(0..0, |rng, &x| {
                rng.start = rng.end;
                rng.end += x;
                Some(rng.clone())
            })
            .enumerate()
            .map(|(ind, rng)| {
                let boundaries = &self.boundaries[rng.start + ind..rng.end + ind + 1];
                self.annotation[rng]
                    .iter()
                    .enumerate()
                    .map(move |(i, anno)| (boundaries[i], boundaries[i + 1], anno))
            })
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.hitlen.clear();
        self.boundaries.clear();
        for anvec in self.annotation.iter_mut() {
            anvec.clear();
        }
    }

    pub fn reset(&mut self) -> &mut Self {
        self.clear();
        self
    }

    pub fn is_empty(&self) -> bool {
        self.hitlen.is_empty()
    }

    pub fn len(&self) -> usize {
        self.hitlen.len()
    }
}

impl<Idx: PrimInt, T: Eq + Hash + Clone> Default for Steps<Idx, T> {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interval_tree::overlap::elements::{tests::add_overlaps, Elements};

    fn assert_steps<'a>(
        steps: &'a Steps<usize, &str>,
        expected: &'a [impl AsRef<[(usize, usize, Vec<&'a str>)]>],
    ) {
        let mut iters = steps.iter().collect::<Vec<_>>();
        assert_eq!(iters.len(), expected.len());

        for (exp, iter) in expected.iter().zip(iters.iter_mut()) {
            let iter: Vec<_> = iter.map(|(st, en, anno)| (st, en, anno.clone())).collect();
            let exp: Vec<_> = exp
                .as_ref()
                .iter()
                .map(|(st, en, anno)| {
                    (
                        *st,
                        *en,
                        anno.into_iter().map(|&x| x).collect::<HashSet<_>>(),
                    )
                })
                .collect();

            assert_eq!(iter, exp);
        }
    }

    #[test]
    fn test_steps_empty() {
        let mut overlap = Elements::empty();
        add_overlaps(&mut overlap, &vec![vec![]]);

        let mut steps = Steps::empty();
        steps.build([Interval::new(2, 8).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &[vec![(2, 8, vec![])]]);
    }
    #[test]
    fn test_steps_single_1() {
        let mut overlap = Elements::empty();
        add_overlaps(
            &mut overlap,
            &vec![vec![
                ((1..3).try_into().unwrap(), "a"),
                ((4..6).try_into().unwrap(), "b"),
                ((7..9).try_into().unwrap(), "c"),
            ]],
        );

        let mut steps = Steps::empty();
        let expected = [vec![
            (0, 1, vec![]),
            (1, 3, vec!["a"]),
            (3, 4, vec![]),
            (4, 6, vec!["b"]),
            (6, 7, vec![]),
            (7, 9, vec!["c"]),
            (9, 10, vec![]),
        ]];

        // Option 1 - Query interval covers all intervals completely
        steps.build([Interval::new(0, 10).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &expected[..]);

        // Option 2 - Query interval envelops all intervals
        steps.build([Interval::new(1, 9).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &[expected[0][1..6].to_vec()]);

        // Option 3 - Query interval intersects all intervals **somehow**
        steps.build([Interval::new(1, 7).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &[expected[0][1..5].to_vec()]);

        steps.build([Interval::new(2, 8).unwrap()].iter().zip(overlap.iter()));
        assert_steps(
            &steps,
            &[vec![
                (2, 3, vec!["a"]),
                (3, 4, vec![]),
                (4, 6, vec!["b"]),
                (6, 7, vec![]),
                (7, 8, vec!["c"]),
            ]],
        );
    }

    #[test]
    fn test_steps_single_2() {
        let mut overlap = Elements::empty();
        add_overlaps(
            &mut overlap,
            &vec![vec![
                ((0..2).try_into().unwrap(), "a"),
                ((8..10).try_into().unwrap(), "e"),
                ((2..4).try_into().unwrap(), "b"),
                ((4..6).try_into().unwrap(), "c"),
                ((6..8).try_into().unwrap(), "d"),
            ]],
        );

        let mut steps = Steps::empty();
        let expected = [vec![
            (0, 2, vec!["a"]),
            (2, 4, vec!["b"]),
            (4, 6, vec!["c"]),
            (6, 8, vec!["d"]),
            (8, 10, vec!["e"]),
            (10, 12, vec![]),
        ]];

        // Option 1 - Query interval covers all intervals completely
        steps.build([Interval::new(0, 10).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &[expected[0][..5].to_vec()]);

        // Option 2 - Query interval envelops all intervals
        steps.build([Interval::new(0, 12).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &expected);

        // Option 3 - Query interval intersects all intervals **somehow**
        steps.build([Interval::new(1, 9).unwrap()].iter().zip(overlap.iter()));
        assert_steps(
            &steps,
            &[vec![
                (1, 2, vec!["a"]),
                (2, 4, vec!["b"]),
                (4, 6, vec!["c"]),
                (6, 8, vec!["d"]),
                (8, 9, vec!["e"]),
            ]],
        );
    }

    #[test]
    fn test_steps_single_3() {
        let mut overlap = Elements::empty();
        add_overlaps(
            &mut overlap,
            &vec![vec![
                ((1..9).try_into().unwrap(), "a"),
                ((2..8).try_into().unwrap(), "b"),
                ((3..7).try_into().unwrap(), "c"),
            ]],
        );

        let mut steps = Steps::empty();
        let expected = [vec![
            (0, 1, vec![]),
            (1, 2, vec!["a"]),
            (2, 3, vec!["a", "b"]),
            (3, 7, vec!["a", "b", "c"]),
            (7, 8, vec!["a", "b"]),
            (8, 9, vec!["a"]),
            (9, 13, vec![]),
        ]];

        // Option 1 - Query interval covers all intervals completely
        steps.build([Interval::new(0, 13).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &expected);

        steps.build([Interval::new(1, 13).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &[expected[0][1..].to_vec()]);

        // Option 2 - Query interval envelops all intervals
        steps.build([Interval::new(1, 9).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &[expected[0][1..6].to_vec()]);

        // Option 3 - Query interval intersects all intervals **somehow**
        steps.build([Interval::new(2, 8).unwrap()].iter().zip(overlap.iter()));
        assert_steps(
            &steps,
            &[vec![
                (2, 3, vec!["a", "b"]),
                (3, 7, vec!["a", "b", "c"]),
                (7, 8, vec!["a", "b"]),
            ]],
        );

        steps.build([Interval::new(5, 6).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &[vec![(5, 6, vec!["a", "b", "c"])]]);

        steps.build([Interval::new(5, 8).unwrap()].iter().zip(overlap.iter()));
        assert_steps(
            &steps,
            &[vec![(5, 7, vec!["a", "b", "c"]), (7, 8, vec!["a", "b"])]],
        );
    }

    #[test]
    fn test_steps_single_4() {
        let mut overlap = Elements::empty();
        add_overlaps(
            &mut overlap,
            &vec![vec![
                ((1..9).try_into().unwrap(), "a"),
                ((2..4).try_into().unwrap(), "b"),
                ((3..6).try_into().unwrap(), "a"),
            ]],
        );

        let mut steps = Steps::empty();
        let expected = [vec![
            (0, 1, vec![]),
            (1, 2, vec!["a"]),
            (2, 3, vec!["a", "b"]),
            (3, 4, vec!["a", "b"]),
            (4, 6, vec!["a"]),
            (6, 9, vec!["a"]),
            (9, 13, vec![]),
        ]];

        // Option 1 - Query interval covers all intervals completely
        steps.build([Interval::new(0, 13).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &expected);

        steps.build([Interval::new(1, 13).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &[expected[0][1..].to_vec()]);

        // Option 2 - Query interval envelops all intervals
        steps.build([Interval::new(1, 9).unwrap()].iter().zip(overlap.iter()));
        assert_steps(&steps, &[expected[0][1..6].to_vec()]);

        // Option 3 - Query interval intersects all intervals **somehow**
        steps.build([Interval::new(2, 8).unwrap()].iter().zip(overlap.iter()));
        assert_steps(
            &steps,
            &[vec![
                (2, 3, vec!["a", "b"]),
                (3, 4, vec!["a", "b"]),
                (4, 6, vec!["a"]),
                (6, 8, vec!["a"]),
            ]],
        );
    }

    #[test]
    fn test_steps_multi_1() {
        let mut overlap = Elements::empty();
        add_overlaps(
            &mut overlap,
            &vec![
                vec![],
                vec![
                    ((1..9).try_into().unwrap(), "a1"),
                    ((2..8).try_into().unwrap(), "b1"),
                    ((3..7).try_into().unwrap(), "c1"),
                ],
                vec![],
                vec![],
                vec![
                    ((1..3).try_into().unwrap(), "a2"),
                    ((4..6).try_into().unwrap(), "b2"),
                    ((7..9).try_into().unwrap(), "c2"),
                ],
                vec![
                    ((1..9).try_into().unwrap(), "a3"),
                    ((2..8).try_into().unwrap(), "b3"),
                    ((3..5).try_into().unwrap(), "c3"),
                    ((4..12).try_into().unwrap(), "a3"),
                ],
                vec![],
            ],
        );

        let mut steps = Steps::empty();
        let expected = [
            vec![(0, 10, vec![])],
            vec![
                (0, 1, vec![]),
                (1, 2, vec!["a1"]),
                (2, 3, vec!["a1", "b1"]),
                (3, 7, vec!["a1", "b1", "c1"]),
                (7, 8, vec!["a1", "b1"]),
                (8, 9, vec!["a1"]),
                (9, 13, vec![]),
            ],
            vec![(1, 2, vec![])],
            vec![(65, 80, vec![])],
            vec![
                (0, 1, vec![]),
                (1, 3, vec!["a2"]),
                (3, 4, vec![]),
                (4, 6, vec!["b2"]),
                (6, 7, vec![]),
                (7, 9, vec!["c2"]),
                (9, 10, vec![]),
            ],
            vec![
                (0, 1, vec![]),
                (1, 2, vec!["a3"]),
                (2, 3, vec!["a3", "b3"]),
                (3, 4, vec!["a3", "b3", "c3"]),
                (4, 5, vec!["a3", "b3", "c3"]),
                (5, 8, vec!["a3", "b3"]),
                (8, 9, vec!["a3"]),
                (9, 12, vec!["a3"]),
                (12, 15, vec![]),
            ],
            vec![(100, 101, vec![])],
        ];

        // Option 1 - Query interval covers all intervals completely
        steps.build(
            [
                Interval::new(0, 10).unwrap(),
                Interval::new(0, 13).unwrap(),
                Interval::new(1, 2).unwrap(),
                Interval::new(65, 80).unwrap(),
                Interval::new(0, 10).unwrap(),
                Interval::new(0, 15).unwrap(),
                Interval::new(100, 101).unwrap(),
            ]
            .iter()
            .zip(overlap.iter()),
        );
        assert_steps(&steps, &expected);
    }
}
