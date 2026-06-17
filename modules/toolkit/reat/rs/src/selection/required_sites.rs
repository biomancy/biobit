#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use std::collections::BTreeMap;

use biobit_collections_rs::interval_tree::Bits;
use biobit_core_rs::loc::{Interval, IntervalOp, Orientation};
use biobit_core_rs::num::PrimUInt;
use eyre::{Result, bail};

use crate::dna::Reference;
use crate::pileup::DensePileup;
use crate::selection::{Selection, Selector};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RequiredSites<SeqId: Ord = String, Idx: PrimUInt = u64> {
    index: BTreeMap<(SeqId, Orientation), Bits<Idx, ()>>,
}

impl<SeqId, Idx> RequiredSites<SeqId, Idx>
where
    SeqId: Ord,
    Idx: PrimUInt,
{
    pub fn new<Required, Intervals>(required: Required) -> Self
    where
        Required: IntoIterator<Item = (SeqId, Orientation, Intervals)>,
        Intervals: IntoIterator<Item = Interval<Idx>>,
    {
        let mut grouped = BTreeMap::<(SeqId, Orientation), Vec<Interval<Idx>>>::new();
        for (seqid, orientation, intervals) in required {
            grouped
                .entry((seqid, orientation))
                .or_default()
                .extend(intervals);
        }

        let index = grouped
            .into_iter()
            .filter_map(|(key, mut intervals)| {
                if intervals.is_empty() {
                    return None;
                }
                let intervals = Interval::merge(&mut intervals);
                let records = intervals.into_iter().map(|interval| (interval, ()));
                Some((key, Bits::new(records)))
            })
            .collect();

        Self { index }
    }

    #[inline]
    pub fn index(&self) -> &BTreeMap<(SeqId, Orientation), Bits<Idx, ()>> {
        &self.index
    }
}

impl<SeqId, Idx> Default for RequiredSites<SeqId, Idx>
where
    SeqId: Ord,
    Idx: PrimUInt,
{
    fn default() -> Self {
        Self {
            index: BTreeMap::new(),
        }
    }
}

impl<SeqId, Idx, Cnts> Selector<SeqId, Idx, Cnts> for RequiredSites<SeqId, Idx>
where
    SeqId: Clone + Ord,
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    fn select(
        &self,
        seqid: &SeqId,
        orientation: Orientation,
        pileup: &DensePileup<Idx, Cnts>,
        _reference: &[Reference],
        selection: &mut Selection,
    ) -> Result<()> {
        let key = (seqid.clone(), orientation);
        let Some(required) = self.index.get(&key) else {
            return Ok(());
        };
        let overlaps = required.query(*pileup.interval()).map(|(i, _)| i);
        for overlap in overlaps.into_iter() {
            // Overlaps aren't guaranteed to be fully contained within the pileup interval, so we need to compute the intersection
            let Some(intersection) = overlap.intersection(pileup.interval()) else {
                bail!("Overlap calculated by Bits should intersect with pileup interval");
            };
            let start = (intersection.start() - pileup.interval().start())
                .to_usize()
                .expect("intersection start should be within pileup interval");
            let end = (intersection.end() - pileup.interval().start())
                .to_usize()
                .expect("intersection end should be within pileup interval");
            for offset in start..end {
                if !pileup.counts().site(offset).coverage().is_zero() {
                    selection.select(offset)
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::pileup::Pileup;

    use super::*;

    fn dense() -> Result<DensePileup<u64, u32>> {
        DensePileup::new(
            Interval::new(10_u64, 15)?,
            Pileup::<u32>::new(
                vec![1, 2, 0, 4, 5],
                vec![0, 0, 0, 0, 0],
                vec![0, 0, 0, 0, 0],
                vec![0, 0, 0, 0, 0],
                vec![0, 0, 0, 0, 0],
                vec![0, 0, 0, 0, 0],
            )?,
        )
    }

    #[test]
    fn builds_index_by_seqid_and_orientation() -> Result<()> {
        let selector = RequiredSites::new([
            (
                "chr1",
                Orientation::Forward,
                vec![Interval::new(12_u64, 14)?],
            ),
            (
                "chr1",
                Orientation::Reverse,
                vec![Interval::new(20_u64, 21)?],
            ),
        ]);

        assert_eq!(selector.index.len(), 2);
        assert!(selector.index.contains_key(&("chr1", Orientation::Forward)));
        assert!(selector.index.contains_key(&("chr1", Orientation::Reverse)));
        Ok(())
    }

    #[test]
    fn selects_required_sites_for_matching_key_when_covered() -> Result<()> {
        let selector = RequiredSites::new([
            (
                "chr1",
                Orientation::Forward,
                vec![Interval::new(12_u64, 16)?, Interval::new(14, 15)?],
            ),
            (
                "chr1",
                Orientation::Reverse,
                vec![Interval::new(13_u64, 15)?],
            ),
        ]);
        let dense = dense()?;
        let mut selection = Selection::zeros(dense.len());
        let reference = vec![Reference::N; dense.len()];

        selector.select(
            &"chr1",
            Orientation::Forward,
            &dense,
            &reference,
            &mut selection,
        )?;

        // Note, 2 is not selected because it has zero coverage, even though it's within the required interval
        assert_eq!(selection.selected_offsets().collect::<Vec<_>>(), vec![3, 4]);
        Ok(())
    }
}
