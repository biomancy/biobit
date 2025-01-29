use super::interval::{Interval, IntervalOp};
use crate::num::PrimInt;
use derive_getters::Dissolve;
use eyre::{eyre, Report, Result};
use std::fmt::{Debug, Display};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

/// ChainInterval is an ordered sequence of non-overlapping and non-touching half-open genomic intervals.
#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Dissolve)]
pub struct ChainInterval<Idx: PrimInt> {
    links: Vec<Interval<Idx>>,
}

impl<Idx: PrimInt> Default for ChainInterval<Idx> {
    fn default() -> Self {
        Self {
            links: vec![Interval::default()],
        }
    }
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

    pub fn start(&self) -> Idx {
        // Safe to unwrap because ChainInterval can't be empty
        unsafe { self.links.first().unwrap_unchecked().start() }
    }

    pub fn end(&self) -> Idx {
        // Safe to unwrap because ChainInterval can't be empty
        unsafe { self.links.last().unwrap_unchecked().end() }
    }

    pub fn cast<T: PrimInt>(&self) -> Option<ChainInterval<T>> {
        let mut links = Vec::with_capacity(self.links.len());
        for link in &self.links {
            match link.cast() {
                Some(link) => links.push(link),
                None => return None,
            }
        }
        Some(ChainInterval { links })
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

// impl<T, R: PartialEq<T>> PartialEq<Mapping<T>> for Mapping<R> {
//     fn eq(&self, other: &Mapping<T>) -> bool {
//         match (self, other) {
//             (Mapping::Complete(x), Mapping::Complete(y)) => x == y,
//             (Mapping::Truncated(x), Mapping::Truncated(y)) => x == y,
//             (Mapping::None, Mapping::None) => true,
//             _ => false,
//         }
//     }
// }

impl<Idx: PrimInt> From<ChainInterval<Idx>> for Vec<Interval<Idx>> {
    fn from(chain: ChainInterval<Idx>) -> Self {
        chain.links
    }
}

impl<Idx: PrimInt, T> PartialEq<[T]> for ChainInterval<Idx>
where
    Interval<Idx>: PartialEq<T>,
{
    fn eq(&self, other: &[T]) -> bool {
        self.links == *other
    }
}

impl<Idx: PrimInt, T> PartialEq<Vec<T>> for ChainInterval<Idx>
where
    Interval<Idx>: PartialEq<T>,
{
    fn eq(&self, other: &Vec<T>) -> bool {
        self.links == *other
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
