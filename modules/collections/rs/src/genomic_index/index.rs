use std::collections::HashMap;

use derive_getters::Dissolve;

use biobit_core_rs::loc::{Contig, LikeLocus, Locus, Orientation};

use crate::genomic_index::OverlapIntervals;
use crate::interval_tree::{IntervalTree, IntervalTreeElement, IntervalTreeLendingIterator};

#[derive(Clone, PartialEq, Eq, Debug, Dissolve)]
pub struct GenomicIndex<Ctg: Contig, IT: IntervalTree> {
    itrees: HashMap<Ctg, HashMap<Orientation, IT>>,
}

impl<Ctg: Contig, IT: IntervalTree> Default for GenomicIndex<Ctg, IT> {
    fn default() -> Self { Self { itrees: HashMap::new() } }
}

impl<Ctg: Contig, IT: IntervalTree> GenomicIndex<Ctg, IT> {
    pub fn new() -> Self { Self::default() }

    pub fn set(&mut self, contig: Ctg, orientation: Orientation, index: IT) {
        let mut map = HashMap::new();
        map.insert(orientation, index);

        self.itrees.insert(contig, map);
    }

    pub fn overlap<'a, 'b>(
        &'a self, locus: Locus<Ctg, IT::Idx>, query: OverlapIntervals<'b, Ctg, IT::Idx, IT::Value>,
    ) -> OverlapIntervals<'a, Ctg, IT::Idx, IT::Value> {
        let index = self.itrees
            .get(locus.contig())
            .map(|m| m.get(&locus.orientation()))
            .flatten();
        match index {
            None => { query.reset(locus, std::iter::empty()) }
            Some(tree) => {
                let mut a = Vec::new();
                let mut iter = tree.intersection(locus.interval());
                while let Some(element) = iter.next() {
                    a.push((element.interval().clone(), element.value()));
                }
                query.reset(locus, a.into_iter())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use biobit_core_rs::loc::Interval;

    use crate::interval_tree::{IntervalTreeBuilder, LapperIntervalTreeBuilder};

    use super::*;

    #[test]
    fn test_genomic_index() {
        let itree = LapperIntervalTreeBuilder::<usize, usize>::new()
            .add(&Interval::new(0, 10).unwrap(), 1)
            .add(&Interval::new(5, 15).unwrap(), 2)
            .build();
        let mut index: GenomicIndex<&str, _> = GenomicIndex::new();
        index.set("ctg1", Orientation::Forward, itree);

        let overlaps = index.overlap(
            ("ctg1", 0..10, Orientation::Forward).try_into().unwrap(),
            OverlapIntervals::new(),
        );

        assert_eq!(
            overlaps.intervals(),
            [Interval::new(0, 10).unwrap(), Interval::new(5, 15).unwrap()]
        );
        assert_eq!(
            overlaps.annotations(),
            [&1, &2]
        );

        // Opposite orientation -> no overlaps
        let overlaps = index.overlap(
            ("ctg1", 0..10, Orientation::Reverse).try_into().unwrap(),
            overlaps,
        );
        assert!(overlaps.intervals().is_empty());
        assert!(overlaps.annotations().is_empty());
    }
}