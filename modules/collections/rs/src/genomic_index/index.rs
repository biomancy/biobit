use std::collections::HashMap;

use ::higher_kinded_types::prelude::*;
use derive_getters::Dissolve;

use biobit_core_rs::loc::{Contig, Orientation, Segment};
use biobit_core_rs::LendingIterator;

use crate::genomic_index::OverlapSegments;
use crate::interval_tree::{ITree, TreeRecord};

#[derive(Clone, PartialEq, Eq, Debug, Dissolve)]
pub struct GenomicIndex<Ctg: Contig, IT: ITree> {
    itrees: HashMap<Ctg, HashMap<Orientation, IT>>,
}

impl<Ctg: Contig, IT: ITree> Default for GenomicIndex<Ctg, IT> {
    fn default() -> Self {
        Self {
            itrees: HashMap::new(),
        }
    }
}

impl<Ctg: Contig, IT: ITree> GenomicIndex<Ctg, IT> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, contig: Ctg, orientation: Orientation, index: IT) {
        let mut map = HashMap::new();
        map.insert(orientation, index);

        self.itrees.insert(contig, map);
    }

    pub fn overlap<'a, 'b, 'c>(
        &'a self,
        ctg: &Ctg,
        orientation: Orientation,
        segments: impl Iterator<Item = &'c Segment<IT::Idx>>,
        buffer: OverlapSegments<'b, IT::Idx, IT::Value>,
    ) -> OverlapSegments<'a, IT::Idx, IT::Value>
    where
        <IT as ITree>::Idx: 'c,
        <<IT as ITree>::Iter as ForLt>::Of<'a>: LendingIterator,
        for<'iter> <<<<IT as ITree>::Iter as ForLt>::Of<'a> as LendingIterator>::Item as ForLt>::Of<'iter>:
            TreeRecord<'a, 'iter, Idx = IT::Idx, Value = IT::Value>,
    {
        let index = self.itrees.get(ctg).map(|m| m.get(&orientation)).flatten();
        let mut buffer = buffer.reset();

        if let Some(tree) = index {
            for segment in segments {
                let mut adder = buffer.add();
                let mut iter = tree.intersection(segment);
                while let Some(element) = iter.next() {
                    adder.add(element.interval().clone(), element.value());
                }
                adder.finish();
            }
        };
        buffer
    }
}

#[cfg(test)]
mod tests {
    use biobit_core_rs::loc::Segment;

    use crate::interval_tree::{Builder, LapperBuilder};

    use super::*;

    #[test]
    fn test_genomic_index() {
        let itree = LapperBuilder::<usize, usize>::new()
            .add(&Segment::new(0, 10).unwrap(), 1)
            .add(&Segment::new(5, 15).unwrap(), 2)
            .build();
        let mut index: GenomicIndex<&str, _> = GenomicIndex::new();
        index.set("ctg1", Orientation::Forward, itree);

        let query = vec![Segment::new(0, 10).unwrap()];
        let overlaps = index.overlap(
            &"ctg1",
            Orientation::Forward,
            query.iter(),
            OverlapSegments::new(),
        );
        assert_eq!(overlaps.len(), 1);
        assert_eq!(
            overlaps.segments().collect::<Vec<_>>(),
            [&[Segment::new(0, 10).unwrap(), Segment::new(5, 15).unwrap()]]
        );
        assert_eq!(overlaps.annotations().collect::<Vec<_>>(), [&[&1, &2]]);

        // Opposite orientation -> no overlaps
        let overlaps = index.overlap(
            &"ctg1",
            Orientation::Reverse,
            query.iter(),
            OverlapSegments::new(),
        );
        assert_eq!(overlaps.len(), 0);
        assert!(overlaps.segments().collect::<Vec<_>>().is_empty());
        assert!(overlaps.annotations().collect::<Vec<_>>().is_empty());

        // Multiple queries with multiple overlaps
        let query = vec![
            Segment::new(0, 2).unwrap(),
            Segment::new(5, 7).unwrap(),
            Segment::new(10, 12).unwrap(),
        ];
        let overlaps = index.overlap(
            &"ctg1",
            Orientation::Forward,
            query.iter(),
            OverlapSegments::new(),
        );
        assert_eq!(overlaps.len(), 3);
        assert_eq!(
            overlaps.segments().collect::<Vec<_>>(),
            [
                vec![Segment::new(0, 10).unwrap()],
                vec![Segment::new(0, 10).unwrap(), Segment::new(5, 15).unwrap()],
                vec![Segment::new(5, 15).unwrap()]
            ]
        );
        assert_eq!(
            overlaps.annotations().collect::<Vec<_>>(),
            [vec![&1], vec![&1, &2], vec![&2]]
        );

        // Multiple queries with no overlaps
        let overlaps = index.overlap(
            &"ctg1",
            Orientation::Dual,
            query.iter(),
            OverlapSegments::new(),
        );
        assert_eq!(overlaps.len(), 0);
        assert!(overlaps.segments().collect::<Vec<_>>().is_empty());
        assert!(overlaps.annotations().collect::<Vec<_>>().is_empty());
    }
}
