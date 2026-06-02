use std::collections::BTreeMap;

use biobit_core_rs::loc::Orientation;
use biobit_core_rs::num::PrimUInt;
use eyre::Result;

use crate::pileup::SparsePileup;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SelectedPileup<SeqId = String, Idx: PrimUInt = u64, Cnts: PrimUInt = u32, Tag = ()> {
    pub tag: Tag,
    pub pileups: BTreeMap<(SeqId, Orientation), SparsePileup<SeqId, Idx, Cnts>>,
}

impl<SeqId, Idx, Cnts, Tag> SelectedPileup<SeqId, Idx, Cnts, Tag>
where
    SeqId: Clone + Ord,
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn new(tag: Tag, pileups: Vec<SparsePileup<SeqId, Idx, Cnts>>) -> Result<Self> {
        let mut grouped = BTreeMap::<(SeqId, Orientation), Vec<_>>::new();
        for pileup in pileups {
            grouped
                .entry((pileup.seqid.clone(), pileup.orientation))
                .or_default()
                .push(pileup);
        }

        let pileups = grouped
            .into_iter()
            .map(|(key, mut chunks)| {
                chunks.sort_by_key(|chunk| chunk.interval());
                let pileup = SparsePileup::from_distinct_chunks(&chunks)?;
                Ok((key, pileup))
            })
            .collect::<Result<_>>()?;

        Ok(Self { tag, pileups })
    }

    pub fn empty(tag: Tag) -> Self {
        Self {
            tag,
            pileups: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use biobit_core_rs::loc::{Interval, Orientation};

    use super::*;
    use crate::pileup::Pileup;

    fn sparse(
        seqid: &str,
        orientation: Orientation,
        positions: Vec<u64>,
        a: Vec<u32>,
    ) -> Result<SparsePileup<String, u64, u32>> {
        let len = positions.len();
        SparsePileup::new(
            seqid.to_string(),
            orientation,
            positions,
            Pileup::new(
                a,
                vec![0; len],
                vec![1; len],
                vec![2; len],
                vec![3; len],
                vec![4; len],
            )?,
        )
    }

    #[test]
    fn groups_and_merges_pileups_by_seqid_and_orientation() -> Result<()> {
        let result = SelectedPileup::new(
            "sample",
            vec![
                sparse("chr1", Orientation::Reverse, vec![20], vec![2])?,
                sparse("chr2", Orientation::Forward, vec![5], vec![3])?,
                sparse("chr1", Orientation::Reverse, vec![10], vec![1])?,
            ],
        )?;

        assert_eq!(result.tag, "sample");
        assert_eq!(result.pileups.len(), 2);

        let chr1 = result
            .pileups
            .get(&("chr1".to_string(), Orientation::Reverse))
            .unwrap();
        assert_eq!(chr1.interval(), Interval::new(10, 21).unwrap());
        assert_eq!(chr1.positions(), &[10, 20]);
        assert_eq!(chr1.counts().a(), &[1, 2]);

        let chr2 = result
            .pileups
            .get(&("chr2".to_string(), Orientation::Forward))
            .unwrap();
        assert_eq!(chr2.positions(), &[5]);
        assert_eq!(chr2.counts().a(), &[3]);
        Ok(())
    }

    #[test]
    fn empty_result_contains_no_pileups() {
        let result = SelectedPileup::<String, u64, u32, _>::empty("sample");
        assert_eq!(result.tag, "sample");
        assert!(result.pileups.is_empty());
    }
}
