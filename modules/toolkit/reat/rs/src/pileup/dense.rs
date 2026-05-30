use biobit_core_rs::loc::{Interval, IntervalOp, Orientation};
use biobit_core_rs::num::PrimUInt;
use eyre::{Result, ensure};

use super::{Pileup, Site};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DensePileup<SeqId = String, Idx: PrimUInt = u64, Cnts: PrimUInt = u32> {
    pub seqid: SeqId,
    pub orientation: Orientation,
    interval: Interval<Idx>,
    counts: Pileup<Cnts>,
}

#[allow(clippy::len_without_is_empty)]
impl<SeqId, Idx: PrimUInt, Cnts: PrimUInt> DensePileup<SeqId, Idx, Cnts> {
    pub fn new(
        seqid: SeqId,
        interval: Interval<Idx>,
        orientation: Orientation,
        counts: Pileup<Cnts>,
    ) -> Result<Self> {
        let len = interval
            .len()
            .to_usize()
            .ok_or_else(|| eyre::eyre!("Pileup interval length does not fit into usize"))?;
        ensure!(
            counts.len() == len,
            "Pileup counts length does not match pileup interval length"
        );

        Ok(Self {
            seqid,
            interval,
            orientation,
            counts,
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.counts.len()
    }

    #[inline]
    pub fn interval(&self) -> Interval<Idx> {
        self.interval
    }

    #[inline]
    pub fn counts(&self) -> &Pileup<Cnts> {
        &self.counts
    }

    #[inline]
    pub fn counts_mut(&mut self) -> &mut Pileup<Cnts> {
        &mut self.counts
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Idx, Site<'_, Cnts>)> + '_ {
        let mut position = self.interval.start();
        self.counts.iter().map(move |site| {
            let current = position;
            position = position + Idx::one();
            (current, site)
        })
    }

    pub fn reset(
        &mut self,
        seqid: SeqId,
        interval: Interval<Idx>,
        orientation: Orientation,
    ) -> Result<()> {
        let length = interval
            .len()
            .to_usize()
            .ok_or_else(|| eyre::eyre!("Pileup interval length does not fit into usize"))?;

        self.seqid = seqid;
        self.interval = interval;
        self.orientation = orientation;
        self.counts.reset(length);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::{Observed, Reference};

    #[test]
    fn validates_lengths() -> Result<()> {
        let dense = DensePileup::new(
            "chr1".to_string(),
            Interval::new(10_u64, 12).unwrap(),
            Orientation::Forward,
            Pileup::<u32>::zeros(2),
        )?;

        assert_eq!(dense.len(), 2);
        assert_eq!(dense.seqid, "chr1");
        assert_eq!(dense.orientation, Orientation::Forward);
        assert_eq!(dense.interval(), Interval::new(10_u64, 12).unwrap());
        Ok(())
    }

    #[test]
    fn rejects_mismatched_counts_length() {
        assert!(
            DensePileup::new(
                "chr1",
                Interval::new(10_u64, 12).unwrap(),
                Orientation::Forward,
                Pileup::<u32>::zeros(1),
            )
            .is_err()
        );
    }

    #[test]
    fn reset_updates_metadata_and_resizes_contents() -> Result<()> {
        let mut dense = DensePileup::new(
            "chr1".to_string(),
            Interval::new(10_u64, 12).unwrap(),
            Orientation::Forward,
            Pileup::<u32>::zeros(2),
        )?;
        dense.counts_mut()[Observed::A][0] = 5;

        dense.reset(
            "chr2".to_string(),
            Interval::new(20_u64, 24).unwrap(),
            Orientation::Reverse,
        )?;

        assert_eq!(dense.seqid, "chr2");
        assert_eq!(dense.orientation, Orientation::Reverse);
        assert_eq!(dense.interval(), Interval::new(20_u64, 24).unwrap());
        assert_eq!(dense.counts().a(), &[0, 0, 0, 0]);
        Ok(())
    }

    #[test]
    fn iterates_sites() -> Result<()> {
        let dense = DensePileup::new(
            "chr1",
            Interval::new(10_u64, 13).unwrap(),
            Orientation::Forward,
            Pileup::<u32>::zeros(3),
        )?;

        let mut sites = dense.iter();
        assert_eq!(sites.size_hint(), (3, Some(3)));

        let (position, site) = sites.next().unwrap();
        assert_eq!(site.offset(), 0);
        assert_eq!(position, 10);
        assert_eq!(sites.size_hint(), (2, Some(2)));
        assert_eq!(
            dense
                .iter()
                .map(|(position, _site)| position)
                .collect::<Vec<_>>(),
            vec![10, 11, 12]
        );
        Ok(())
    }

    #[test]
    fn site_calculates_counts_coverage_and_mismatches() -> Result<()> {
        let dense = DensePileup::new(
            "chr1",
            Interval::new(10_u64, 15).unwrap(),
            Orientation::Forward,
            Pileup::<u32>::new(
                vec![2, 1, 1, 1, 1],
                vec![1, 2, 1, 1, 1],
                vec![1, 1, 2, 1, 1],
                vec![1, 1, 1, 2, 1],
                vec![1, 1, 1, 1, 2],
                vec![1, 1, 1, 1, 1],
            )?,
        )?;

        let sites = dense.iter().collect::<Vec<_>>();

        assert_eq!(dense.len(), 5);
        assert_eq!(sites[0].1.offset(), 0);
        assert_eq!(sites[0].0, 10);
        assert_eq!(*sites[0].1.a(), 2);
        assert_eq!(sites[0].1.coverage(), 7);
        assert_eq!(sites[0].1.matches(Reference::A), 2);
        assert_eq!(sites[0].1.mismatches(Reference::A), 5);
        assert!(sites[0].1.is_covered());
        let reference = [
            Reference::A,
            Reference::C,
            Reference::G,
            Reference::T,
            Reference::N,
        ];
        assert_eq!(
            sites
                .iter()
                .zip(reference)
                .map(|((_position, site), reference)| site.mismatches(reference))
                .collect::<Vec<_>>(),
            vec![5, 5, 5, 5, 5]
        );
        Ok(())
    }
}
