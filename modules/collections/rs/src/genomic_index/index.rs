use std::collections::HashMap;

use derive_getters::Dissolve;

use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{Contig, Orientation, Segment};

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
        self.itrees
            .entry(contig)
            .or_default()
            .insert(orientation, index);
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
    {
        let index = self.itrees.get(ctg)
            .and_then(|m| m.get(&orientation));
        let mut buffer = buffer.reset();

        if let Some(tree) = index {
            for segment in segments {
                let mut adder = buffer.add();
                let mut iter = tree.intersection(segment);
                while let Some(element) = iter.next() {
                    adder.add(*element.interval(), element.value());
                }
                adder.finish();
            }
        } else {
            for _ in segments {
                buffer.add().finish();
            }
        }
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
            OverlapSegments::default(),
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
            OverlapSegments::default(),
        );
        assert_eq!(overlaps.len(), 1);
        assert!(overlaps.segments().all(|x| x.is_empty()));
        assert!(overlaps.annotations().all(|x| x.is_empty()));

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
            OverlapSegments::default(),
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
            OverlapSegments::default(),
        );
        assert_eq!(overlaps.len(), 3);
        assert!(overlaps.segments().all(|x| x.is_empty()));
        assert!(overlaps.annotations().all(|x| x.is_empty()));
    }
}
