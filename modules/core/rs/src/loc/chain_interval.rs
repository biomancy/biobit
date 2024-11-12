use super::interval::{Interval, IntervalOp};
use crate::num::PrimInt;
use derive_getters::Dissolve;
use eyre::{eyre, Report, Result};
use std::fmt::{Debug, Display};
use std::ops::Range;

/// ChainInterval is an ordered sequence of non-overlapping and non-touching half-open genomic intervals.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve)]
pub struct ChainInterval<Idx: PrimInt> {
    links: Vec<Interval<Idx>>,
}

impl<Idx: PrimInt> ChainInterval<Idx> {
    // Note: There are not TryFromIterator in Rust which would be useful here
    pub fn try_from_iter(iterator: impl Iterator<Item = Interval<Idx>>) -> Result<Self> {
        let links: Vec<_> = iterator.collect();
        if links.is_empty() {
            return Err(eyre!("ChainInterval can't be empty"));
        }

        for i in 1..links.len() {
            if links[i - 1].end() >= links[i].start() {
                return Err(eyre!(
                    "Overlapping intervals aren't allowed in ChainInterval"
                ));
            }
        }
        Ok(Self { links })
    }

    pub fn links(&self) -> &[Interval<Idx>] {
        &self.links
    }
}

impl<Idx: PrimInt + Display> Display for ChainInterval<Idx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for link in &self.links[..self.links.len() - 1] {
            write!(f, "{}-", link)?;
        }
        write!(f, "{}", self.links.last().unwrap())
    }
}

impl<Idx, T> TryFrom<Vec<T>> for ChainInterval<Idx>
where
    Idx: PrimInt,
    T: TryInto<Interval<Idx>, Error = Report>,
{
    type Error = Report;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        let iter: Result<Vec<_>, _> = value.into_iter().map(TryInto::try_into).collect();
        match iter {
            Ok(links) => Self::try_from_iter(links.into_iter()),
            Err(err) => Err(err),
        }
    }
}

impl<Idx: PrimInt> From<ChainInterval<Idx>> for Vec<Interval<Idx>> {
    fn from(chain: ChainInterval<Idx>) -> Self {
        chain.links
    }
}

impl<Idx: PrimInt> PartialEq<Vec<Interval<Idx>>> for ChainInterval<Idx> {
    fn eq(&self, other: &Vec<Interval<Idx>>) -> bool {
        self.links == *other
    }
}

impl<Idx: PrimInt> PartialEq<[Interval<Idx>]> for ChainInterval<Idx> {
    fn eq(&self, other: &[Interval<Idx>]) -> bool {
        self.links == *other
    }
}

impl<Idx: PrimInt> PartialEq<[Range<Idx>]> for ChainInterval<Idx> {
    fn eq(&self, other: &[Range<Idx>]) -> bool {
        if self.links.len() != other.len() {
            return false;
        }
        for (link, range) in self.links.iter().zip(other.iter()) {
            if link != range {
                return false;
            }
        }
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_iter() -> Result<()> {
        // Empty ChainInterval is not allowed
        let chain = ChainInterval::try_from_iter(Vec::<Interval<i64>>::new().into_iter());
        assert!(chain.is_err());

        // Overlapping intervals are not allowed
        let links = vec![Interval::new(0, 10)?, Interval::new(5, 15)?];
        let chain = ChainInterval::try_from_iter(links.into_iter());
        assert!(chain.is_err());

        // Unordered intervals are not allowed
        let links = vec![Interval::new(0, 10)?, Interval::new(-5, 0)?];
        let chain = ChainInterval::try_from_iter(links.into_iter());
        assert!(chain.is_err());

        // Touching intervals are not allowed
        let links = vec![Interval::new(0, 10)?, Interval::new(10, 20)?];
        let chain = ChainInterval::try_from_iter(links.into_iter());
        assert!(chain.is_err());

        // Valid ChainInterval
        let links = vec![Interval::new(0, 10)?, Interval::new(11, 20)?];
        let chain = ChainInterval::try_from_iter(links.clone().into_iter())?;
        assert_eq!(chain, ChainInterval { links });

        Ok(())
    }
}
