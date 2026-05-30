use biobit_core_rs::num::PrimUInt;
use eyre::Result;

use crate::dna::Reference;
use crate::pileup::DensePileup;
use crate::selection::{Mismatches, RequiredSites, Selection, Selector};

#[derive(Clone, PartialEq, Debug)]
pub struct RequiredOrMismatches<SeqId = String, Idx: PrimUInt = u64, Cnts: PrimUInt = u32> {
    required: RequiredSites<SeqId, Idx>,
    mismatches: Mismatches<Cnts>,
}

impl<SeqId, Idx, Cnts> RequiredOrMismatches<SeqId, Idx, Cnts>
where
    SeqId: Ord,
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn new(required: RequiredSites<SeqId, Idx>, mismatches: Mismatches<Cnts>) -> Self {
        Self {
            required,
            mismatches,
        }
    }

    #[inline]
    pub fn required(&self) -> &RequiredSites<SeqId, Idx> {
        &self.required
    }

    #[inline]
    pub fn mismatches(&self) -> &Mismatches<Cnts> {
        &self.mismatches
    }
}

impl<SeqId, Idx, Cnts> Default for RequiredOrMismatches<SeqId, Idx, Cnts>
where
    SeqId: Ord,
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    fn default() -> Self {
        Self {
            required: RequiredSites::default(),
            mismatches: Mismatches::default(),
        }
    }
}

impl<SeqId, Idx, Cnts> Selector<SeqId, Idx, Cnts> for RequiredOrMismatches<SeqId, Idx, Cnts>
where
    SeqId: Clone + Ord,
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    fn select(
        &self,
        pileup: &DensePileup<SeqId, Idx, Cnts>,
        reference: &[Reference],
        selection: &mut Selection,
    ) -> Result<()> {
        self.required.select(pileup, reference, selection)?;
        self.mismatches.select(pileup, reference, selection)
    }
}

#[cfg(test)]
mod tests {
    use biobit_core_rs::loc::{Interval, Orientation};

    use crate::dna::Reference;
    use crate::pileup::Pileup;

    use super::*;

    #[test]
    fn selects_required_sites_or_mismatches() -> Result<()> {
        let selector = RequiredOrMismatches::new(
            RequiredSites::new([(
                "chr1",
                Orientation::Forward,
                vec![Interval::new(10_u64, 12)?],
            )]),
            Mismatches::new(10, 0.0, 1)?,
        );
        let dense = DensePileup::new(
            "chr1",
            Interval::new(10_u64, 14)?,
            Orientation::Forward,
            Pileup::<u32>::new(
                vec![1, 2, 1, 1],
                vec![0, 0, 1, 0],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
                vec![0, 11, 0, 0],
                vec![0, 0, 0, 20],
            )?,
        )?;
        let mut selection = Selection::zeros(dense.len());
        let reference = vec![Reference::A; dense.len()];

        selector.select(&dense, &reference, &mut selection)?;

        assert_eq!(
            selection.selected_offsets().collect::<Vec<_>>(),
            vec![0, 1, 3]
        );
        Ok(())
    }

    #[test]
    fn exposes_inner_selectors() -> Result<()> {
        let selector = RequiredOrMismatches::new(
            RequiredSites::new([(
                "chr1",
                Orientation::Forward,
                vec![Interval::new(10_u64, 12)?],
            )]),
            Mismatches::new(2_u32, 0.35, 3)?,
        );

        assert_eq!(selector.required().index().len(), 1);
        assert_eq!(selector.mismatches().minmismatches(), 2);
        assert_eq!(selector.mismatches().minfreq(), 0.35);
        assert_eq!(selector.mismatches().mincov(), 3);
        Ok(())
    }
}
