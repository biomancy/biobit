use biobit_core_rs::loc::Interval;
use biobit_core_rs::num::PrimInt;
use derive_getters::Dissolve;

// Each query can have 0 or more hits which are stored in a single vector
// For i-th query the hits are stored like this:
// start = hitlen[0] + hitlen[1] + ... + hitlen[i-1]
// end = start + hitlen[i]
// intervals[start..end] & annotations[start..end] are the hits for the i-th query
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Dissolve)]
pub struct Elements<Idx: PrimInt, T> {
    intervals: Vec<Interval<Idx>>,
    elements: Vec<T>,
    hitlen: Vec<usize>, // Strictly equal to the number of queries
}

impl<Idx: PrimInt, T> Default for Elements<Idx, T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<Idx: PrimInt, T> Elements<Idx, T> {
    pub const DEFAULT_CAPACITY: usize = 16;

    pub fn empty() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            intervals: Vec::with_capacity(capacity),
            elements: Vec::with_capacity(capacity),
            hitlen: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.intervals.clear();
        self.elements.clear();
        self.hitlen.clear();
    }

    pub fn add(&mut self) -> OverlapSegmentsAddValue<'_, Idx, T> {
        OverlapSegmentsAddValue {
            length: 0,
            buffer: self,
        }
    }

    pub fn intervals(&self) -> impl Iterator<Item = &'_ [Interval<Idx>]> {
        let mut start = 0;
        self.hitlen.iter().map(move |&x| {
            if x == 0 {
                &self.intervals[0..0]
            } else {
                let rng = start..(start + x);
                start += x;
                &self.intervals[rng]
            }
        })
    }
    pub fn annotations(&self) -> impl Iterator<Item = &'_ [T]> {
        let mut start = 0;
        self.hitlen.iter().map(move |x| {
            if *x == 0 {
                &self.elements[0..0]
            } else {
                let rng = start..(start + x);
                start += x;
                &self.elements[rng]
            }
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'_ [Interval<Idx>], &'_ [T])> {
        let mut start = 0;
        self.hitlen.iter().map(move |x| {
            if *x == 0 {
                (&self.intervals[0..0], &self.elements[0..0])
            } else {
                let rng = start..(start + x);
                start += x;
                (&self.intervals[rng.clone()], &self.elements[rng])
            }
        })
    }

    pub fn is_empty(&self) -> bool {
        self.hitlen.is_empty()
    }

    pub fn len(&self) -> usize {
        self.hitlen.len()
    }
}

pub struct OverlapSegmentsAddValue<'a, Idx: PrimInt, T> {
    length: usize,
    buffer: &'a mut Elements<Idx, T>,
}

impl<Idx: PrimInt, T> OverlapSegmentsAddValue<'_, Idx, T> {
    pub fn add(&mut self, interval: Interval<Idx>, annotation: T) {
        self.buffer.intervals.push(interval);
        self.buffer.elements.push(annotation);
        self.length += 1;
    }

    pub fn finish(mut self) {
        self.buffer.hitlen.push(self.length);
        self.length = usize::MAX;
    }
}

impl<Idx: PrimInt, T> Drop for OverlapSegmentsAddValue<'_, Idx, T> {
    fn drop(&mut self) {
        if self.length != usize::MAX {
            self.buffer.hitlen.push(self.length);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn add_overlaps<'a>(
        query: &mut Elements<usize, &'a str>,
        overlaps: &Vec<Vec<(Interval<usize>, &'a str)>>,
    ) {
        for ov in overlaps.into_iter() {
            let mut adder = query.add();
            for (it, anno) in ov.into_iter() {
                adder.add(it.clone(), anno);
            }
            adder.finish();
        }
    }

    fn assert_elements(
        query: &Elements<usize, &str>,
        overlaps: &Vec<Vec<(Interval<usize>, &str)>>,
    ) {
        assert_eq!(query.len(), overlaps.len());

        let slices: Vec<_> = query.iter().collect();
        assert_eq!(slices.len(), overlaps.len());

        for (expected, produced) in overlaps.iter().zip(slices) {
            let (segments, annotations) = produced;
            let (expseg, expanno): (Vec<_>, Vec<_>) = expected.iter().cloned().unzip();

            assert_eq!(segments, expseg);
            assert_eq!(annotations, expanno);
        }
    }

    #[test]
    fn test_overlap_query_single() {
        let data = vec![vec![
            ((1..3).try_into().unwrap(), "a"),
            ((4..6).try_into().unwrap(), "b"),
            ((7..9).try_into().unwrap(), "c"),
        ]];

        let mut query = Elements::empty();
        add_overlaps(&mut query, &data);

        assert_elements(&query, &data);
    }

    #[test]
    fn test_overlap_query_multiple() {
        let data = vec![
            vec![
                ((1..3).try_into().unwrap(), "a"),
                ((4..6).try_into().unwrap(), "b"),
                ((7..9).try_into().unwrap(), "c"),
            ],
            vec![((5..10).try_into().unwrap(), "a")],
            vec![],
            vec![
                ((10..20).try_into().unwrap(), "a10"),
                ((100..101).try_into().unwrap(), "a100"),
            ],
            vec![],
        ];

        let mut query = Elements::empty();
        add_overlaps(&mut query, &data);

        assert_elements(&query, &data);
    }
}
