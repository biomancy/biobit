use crate::loc::{ChainInterval, Interval, IntervalOp};
use crate::num::PrimInt;
use derive_getters::Getters;
use derive_more::{From, Into};

#[derive(Debug, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Mapping<T> {
    Complete(T),
    Truncated(T),
    None,
}

impl<T, R: PartialEq<T>> PartialEq<Mapping<T>> for Mapping<R> {
    fn eq(&self, other: &Mapping<T>) -> bool {
        match (self, other) {
            (Mapping::Complete(x), Mapping::Complete(y)) => x == y,
            (Mapping::Truncated(x), Mapping::Truncated(y)) => x == y,
            (Mapping::None, Mapping::None) => true,
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Getters, From, Into)]
pub struct ChainMap<Idx: PrimInt> {
    fwdlinks: Vec<Interval<Idx>>,
    bwdlinks: Vec<Interval<Idx>>,
}

impl<Idx: PrimInt> ChainMap<Idx> {
    pub fn new(chain: ChainInterval<Idx>) -> Self {
        let mut position = Idx::zero();
        let mut backlinks = Vec::new();
        for it in chain.links() {
            backlinks.push(it << (it.start() - position));
            position = position + it.len();
        }
        let fwdlinks = chain.dissolve();

        Self {
            fwdlinks,
            bwdlinks: backlinks,
        }
    }

    pub fn invmap_interval(
        &self,
        interval: &Interval<Idx>,
        buffer: ChainInterval<Idx>,
    ) -> Mapping<ChainInterval<Idx>> {
        if interval.start() >= self.bwdlinks.last().unwrap().end()
            || interval.end() <= self.bwdlinks.first().unwrap().start()
        {
            return Mapping::None;
        }

        let mut raw = buffer.dissolve();
        raw.clear();

        // Find the first backlink that intersects with the interval
        let (mut cursor, mut mapped) = match self
            .bwdlinks
            .binary_search_by_key(&interval.start(), |x| x.start())
        {
            Ok(x) => (x, Idx::zero()),
            Err(0) => (0, Idx::zero()),
            Err(x) => {
                // The interval starts in the previous block
                let offset = interval.start() - self.bwdlinks[x - 1].start();
                let length = self.bwdlinks[x - 1].end().min(interval.end()) - interval.start();

                raw.push(unsafe {
                    Interval::new(
                        self.fwdlinks[x - 1].start() + offset,
                        self.fwdlinks[x - 1].start() + offset + length,
                    )
                    .unwrap_unchecked()
                });

                (x, length)
            }
        };

        // Add the rest of the blocks to the chain
        while cursor < self.bwdlinks.len() && self.bwdlinks[cursor].start() < interval.end() {
            // Calculate the offset and length of the next block
            let length =
                self.bwdlinks[cursor].end().min(interval.end()) - self.bwdlinks[cursor].start();

            raw.push(unsafe {
                Interval::new(
                    self.fwdlinks[cursor].start(),
                    self.fwdlinks[cursor].start() + length,
                )
                .unwrap_unchecked()
            });

            cursor += 1;
            mapped = mapped + length;
        }

        if mapped == Idx::zero() {
            Mapping::None
        } else if mapped == interval.len() {
            Mapping::Complete(unsafe {
                ChainInterval::try_from_iter(raw.into_iter()).unwrap_unchecked()
            })
        } else {
            Mapping::Truncated(unsafe {
                ChainInterval::try_from_iter(raw.into_iter()).unwrap_unchecked()
            })
        }
    }

    pub fn map_interval(&self, interval: &Interval<Idx>) -> Mapping<Interval<Idx>> {
        // Check if the interval is outside the chain
        if interval.start() >= self.fwdlinks.last().unwrap().end()
            || interval.end() <= self.fwdlinks.first().unwrap().start()
        {
            return Mapping::None;
        }

        let (ind, start, mut end) = match self
            .fwdlinks
            .binary_search_by_key(&interval.start(), |x| x.start())
        {
            // The interval starts at the beginning of a block
            Ok(x) => (x, self.bwdlinks[x].start(), self.bwdlinks[x].start()),
            // The interval starts before the 0th block
            Err(0) => (0, self.bwdlinks[0].start(), self.bwdlinks[0].start()),
            Err(x) => {
                if interval.start() <= self.fwdlinks[x - 1].end() {
                    // The interval starts within a block
                    debug_assert!(interval.start() >= self.fwdlinks[x - 1].start());
                    let len = self.fwdlinks[x - 1].end().min(interval.end()) - interval.start();
                    let offset = interval.start() - self.fwdlinks[x - 1].start();

                    let start = self.bwdlinks[x - 1].start() + offset;
                    let end = start + len;
                    (x, start, end)
                } else if interval.end() <= self.fwdlinks[x].start() {
                    // The interval lies completely in the gap between two blocks
                    return Mapping::None;
                } else {
                    // The intervals starts in the gap between two blocks
                    (x, self.bwdlinks[x].start(), self.bwdlinks[x].start())
                }
            }
        };

        for link in &self.fwdlinks[ind..] {
            if link.start() >= interval.end() {
                break;
            }
            let len = link.end().min(interval.end()) - link.start();
            end = end + len;
        }

        if end == start {
            Mapping::None
        } else if end - start == interval.len() {
            Mapping::Complete(Interval::new(start, end).unwrap())
        } else {
            Mapping::Truncated(Interval::new(start, end).unwrap())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::loc::ChainInterval;
    use eyre::Result;

    #[test]
    fn test_chain_map_single() -> Result<()> {
        let chain = ChainInterval::try_from_iter([Interval::new(10, 20)?].into_iter())?;
        let map = ChainMap::new(chain);

        for (query, expected) in [
            (10..20, Mapping::Complete(0..10)),
            (5..15, Mapping::Truncated(0..5)),
            (15..25, Mapping::Truncated(5..10)),
            (0..10, Mapping::None),
            (20..30, Mapping::None),
        ] {
            let query = Interval::new(query.start, query.end)?;
            assert_eq!(map.map_interval(&query), expected);
        }

        Ok(())
    }

    #[test]
    fn test_chain_map_multiple() -> Result<()> {
        let map = ChainMap::new(ChainInterval::try_from_iter(
            [
                Interval::new(10, 20)?,
                Interval::new(30, 40)?,
                Interval::new(50, 60)?,
            ]
            .into_iter(),
        )?);

        for (query, expected) in [
            // Complete overlaps
            (10..20, Mapping::Complete(0..10)),
            (30..40, Mapping::Complete(10..20)),
            (50..60, Mapping::Complete(20..30)),
            // Truncated overlaps (single)
            (5..15, Mapping::Truncated(0..5)),
            (15..25, Mapping::Truncated(5..10)),
            (25..35, Mapping::Truncated(10..15)),
            (35..45, Mapping::Truncated(15..20)),
            (45..55, Mapping::Truncated(20..25)),
            (55..65, Mapping::Truncated(25..30)),
            // Truncated overlaps (multiple)
            (5..35, Mapping::Truncated(0..15)),
            (39..51, Mapping::Truncated(19..21)),
            (45..65, Mapping::Truncated(20..30)),
            // Truncated overlaps (all)
            (5..65, Mapping::Truncated(0..30)),
            (9..61, Mapping::Truncated(0..30)),
            (10..60, Mapping::Truncated(0..30)),
            // No overlaps
            (0..10, Mapping::None),
            (20..30, Mapping::None),
            (25..26, Mapping::None),
            (40..50, Mapping::None),
            (45..46, Mapping::None),
            (60..70, Mapping::None),
        ] {
            let query = Interval::new(query.start, query.end)?;
            assert_eq!(map.map_interval(&query), expected);
        }
        Ok(())
    }

    #[test]
    fn test_chain_invmap_single() -> Result<()> {
        let map = ChainMap::new(ChainInterval::try_from_iter(
            [Interval::new(910, 920)?].into_iter(),
        )?);

        for (query, expected) in [
            // Complete mappings
            (0..10, Mapping::Complete(vec![910..920])),
            (5..7, Mapping::Complete(vec![915..917])),
            (9..10, Mapping::Complete(vec![919..920])),
            // Truncated mappings
            (5..15, Mapping::Truncated(vec![915..920])),
            (-1..8, Mapping::Truncated(vec![910..918])),
            (-1..3, Mapping::Truncated(vec![910..913])),
            // No mappings
            (-10..0, Mapping::None),
            (15..25, Mapping::None),
        ] {
            let query = Interval::new(query.start, query.end)?;
            assert_eq!(
                map.invmap_interval(&query, ChainInterval::default()),
                expected
            );
        }

        Ok(())
    }

    #[test]
    fn test_chain_invmap_multiple() -> Result<()> {
        let map = ChainMap::new(ChainInterval::try_from_iter(
            [
                Interval::new(10, 20)?,
                Interval::new(30, 40)?,
                Interval::new(50, 60)?,
            ]
            .into_iter(),
        )?);

        for (query, expected) in [
            // Complete mappings (individual)
            (0..10, Mapping::Complete(vec![10..20])),
            (10..20, Mapping::Complete(vec![30..40])),
            (20..30, Mapping::Complete(vec![50..60])),
            // Complete mappings (multi)
            (0..20, Mapping::Complete(vec![10..20, 30..40])),
            (10..30, Mapping::Complete(vec![30..40, 50..60])),
            (0..30, Mapping::Complete(vec![10..20, 30..40, 50..60])),
            (5..29, Mapping::Complete(vec![15..20, 30..40, 50..59])),
            // Truncated mapping
            (-5..65, Mapping::Truncated(vec![10..20, 30..40, 50..60])),
            (-5..21, Mapping::Truncated(vec![10..20, 30..40, 50..51])),
            (19..61, Mapping::Truncated(vec![39..40, 50..60])),
            // No mappings
            (-10..0, Mapping::None),
            (30..70, Mapping::None),
        ] {
            let query = Interval::new(query.start, query.end)?;
            assert_eq!(
                map.invmap_interval(&query, ChainInterval::default()),
                expected
            );
        }

        Ok(())
    }
}
