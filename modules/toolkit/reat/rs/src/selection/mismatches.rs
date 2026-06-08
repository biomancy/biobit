use biobit_core_rs::num::PrimUInt;
#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use eyre::{Result, ensure};

use crate::dna::Reference;
use crate::pileup::DensePileup;
use crate::selection::{Selection, Selector};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Mismatches<Cnts: PrimUInt = u32> {
    minmismatches: Cnts,
    minfreq: f32,
    mincov: Cnts,
}

impl<Cnts: PrimUInt> Mismatches<Cnts> {
    pub fn new(minmismatches: Cnts, minfreq: f32, mincov: Cnts) -> Result<Self> {
        ensure!(
            (0.0..=1.0).contains(&minfreq),
            "minimum mismatch frequency must be between 0 and 1, got {minfreq}"
        );
        Ok(Self {
            minmismatches,
            minfreq,
            mincov,
        })
    }

    #[inline]
    pub fn mincov(&self) -> Cnts {
        self.mincov
    }

    #[inline]
    pub fn minfreq(&self) -> f32 {
        self.minfreq
    }

    #[inline]
    pub fn minmismatches(&self) -> Cnts {
        self.minmismatches
    }

    #[inline]
    fn enough_mismatches(&self, coverage: Cnts, mismatches: Cnts) -> bool {
        coverage >= self.mincov
            && mismatches >= self.minmismatches
            && mismatch_frequency(coverage, mismatches) >= self.minfreq
    }
}

impl<Cnts: PrimUInt> Default for Mismatches<Cnts> {
    fn default() -> Self {
        Self {
            minfreq: 0.0,
            minmismatches: Cnts::one(),
            mincov: Cnts::one(),
        }
    }
}

impl<SeqId, Idx, Cnts> Selector<SeqId, Idx, Cnts> for Mismatches<Cnts>
where
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    fn select(
        &self,
        pileup: &DensePileup<SeqId, Idx, Cnts>,
        reference: &[Reference],
        selection: &mut Selection,
    ) -> Result<()> {
        debug_assert!(
            reference.len() == pileup.len(),
            "reference length does not match pileup length"
        );

        for (site, reference) in pileup.counts().iter().zip(reference.iter()) {
            let coverage = site.coverage();
            let mismatches = site.mismatches(*reference);
            if self.enough_mismatches(coverage, mismatches) {
                selection.select(site.offset());
            }
        }
        Ok(())
    }
}

fn mismatch_frequency<Cnts: PrimUInt>(coverage: Cnts, mismatches: Cnts) -> f32 {
    if coverage == Cnts::zero() {
        return 0.0;
    }
    let coverage = coverage
        .to_f32()
        .expect("primitive unsigned coverage should fit into f32");
    let mismatches = mismatches
        .to_f32()
        .expect("primitive unsigned mismatches should fit into f32");
    mismatches / coverage
}

#[cfg(test)]
mod tests {
    use biobit_core_rs::loc::{Interval, Orientation};

    use crate::dna::Reference;
    use crate::pileup::Pileup;

    use super::*;

    #[test]
    fn honors_thresholds() -> Result<()> {
        let selector = Mismatches::new(2_u32, 0.5, 3)?;
        let dense = DensePileup::new(
            "chr1",
            Interval::new(10_u64, 13)?,
            Orientation::Forward,
            Pileup::<u32>::new(
                vec![2, 1, 1],
                vec![1, 1, 1],
                vec![0, 0, 0],
                vec![0, 0, 0],
                vec![0, 0, 0],
                vec![0, 1, 0],
            )?,
        )?;
        let mut selection = Selection::zeros(dense.len());
        let reference = vec![Reference::A; dense.len()];

        selector.select(&dense, &reference, &mut selection)?;

        assert_eq!(selection.selected_offsets().collect::<Vec<_>>(), vec![1]);
        Ok(())
    }

    #[test]
    fn matching_observations_are_not_selected() -> Result<()> {
        let dense = DensePileup::new(
            "chr1",
            Interval::new(10_u64, 15)?,
            Orientation::Forward,
            Pileup::<u32>::new(
                vec![1, 0, 0, 0, 0],
                vec![0, 1, 0, 0, 0],
                vec![0, 0, 1, 0, 0],
                vec![0, 0, 0, 1, 0],
                vec![0, 0, 0, 0, 1],
                vec![0, 0, 0, 0, 0],
            )?,
        )?;
        let selector = Mismatches::default();
        let mut selection = Selection::zeros(dense.len());
        let reference = vec![
            Reference::A,
            Reference::C,
            Reference::G,
            Reference::T,
            Reference::N,
        ];

        selector.select(&dense, &reference, &mut selection)?;

        assert!(selection.selected_offsets().next().is_none());
        Ok(())
    }
}
