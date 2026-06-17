use std::collections::BTreeMap;

use biobit_core_rs::loc::Orientation;
use biobit_core_rs::num::PrimUInt;
use eyre::Result;

use crate::task::TaskPileup;
#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SamplePileup<SeqId: Ord = String, Idx: PrimUInt = u64, Cnts: PrimUInt = u32, Tag = ()> {
    pub tag: Tag,
    pub pileups: BTreeMap<(SeqId, Orientation), TaskPileup<Idx, Cnts>>,
}

impl<SeqId, Idx, Cnts, Tag> SamplePileup<SeqId, Idx, Cnts, Tag>
where
    SeqId: Ord,
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn new(
        tag: Tag,
        pileups: BTreeMap<(SeqId, Orientation), Vec<TaskPileup<Idx, Cnts>>>,
    ) -> Result<Self> {
        let pileups = pileups
            .into_iter()
            .map(|(key, chunks)| {
                let pileup = TaskPileup::from_distinct_chunks(chunks)?;
                Ok((key, pileup))
            })
            .collect::<Result<_>>()?;

        Ok(Self { tag, pileups })
    }

    pub fn retag<NewTag>(self, new_tag: NewTag) -> SamplePileup<SeqId, Idx, Cnts, NewTag> {
        SamplePileup {
            tag: new_tag,
            pileups: self.pileups,
        }
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
    use crate::dna::Reference;
    use crate::pileup::{Pileup, SparsePileup};

    fn task_pileup(positions: Vec<u64>, a: Vec<u32>) -> Result<TaskPileup<u64, u32>> {
        let len = positions.len();
        TaskPileup::new(
            SparsePileup::new(
                positions,
                Pileup::new(
                    a,
                    vec![0; len],
                    vec![1; len],
                    vec![2; len],
                    vec![3; len],
                    vec![4; len],
                )?,
            )?,
            vec![Reference::A; len],
        )
    }

    #[test]
    fn groups_and_merges_pileups_by_seqid_and_orientation() -> Result<()> {
        let result = SamplePileup::new(
            "sample",
            BTreeMap::from([
                (
                    ("chr1".to_string(), Orientation::Reverse),
                    vec![
                        task_pileup(vec![20], vec![2])?,
                        task_pileup(vec![10], vec![1])?,
                    ],
                ),
                (
                    ("chr2".to_string(), Orientation::Forward),
                    vec![task_pileup(vec![5], vec![3])?],
                ),
            ]),
        )?;

        assert_eq!(result.tag, "sample");
        assert_eq!(result.pileups.len(), 2);

        let chr1 = result
            .pileups
            .get(&("chr1".to_string(), Orientation::Reverse))
            .unwrap();
        assert_eq!(chr1.interval(), Interval::new(10, 21).unwrap());
        assert_eq!(chr1.pileup().positions(), &[10, 20]);
        assert_eq!(chr1.pileup().counts().a(), &[1, 2]);
        assert_eq!(chr1.reference(), &[Reference::A, Reference::A]);

        let chr2 = result
            .pileups
            .get(&("chr2".to_string(), Orientation::Forward))
            .unwrap();
        assert_eq!(chr2.pileup().positions(), &[5]);
        assert_eq!(chr2.pileup().counts().a(), &[3]);
        Ok(())
    }

    #[test]
    fn empty_result_contains_no_pileups() {
        let result = SamplePileup::<String, u64, u32, _>::empty("sample");
        assert_eq!(result.tag, "sample");
        assert!(result.pileups.is_empty());
    }
}
