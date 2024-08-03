use std::borrow::Borrow;

use itertools::Itertools;

use biobit_core_rs::loc::{AsSegment, Segment};
use biobit_core_rs::num::PrimInt;

use super::inv;

pub struct IndexAnchor<Idx: PrimInt> {
    pub pos: Idx,
    pub repeats: Vec<usize>,
}

pub struct Index<Idx: PrimInt> {
    // Sorted rnas based on their start or end positions
    starts: Vec<IndexAnchor<Idx>>,
    ends: Vec<IndexAnchor<Idx>>,

    // RNA id -> start & end index
    revstart: Vec<usize>,
    revend: Vec<usize>,

    // InvertedRepeat blocks in each InvertedRepeat
    blocks: Vec<Vec<Segment<Idx>>>,
}

impl<Idx: PrimInt> Index<Idx> {
    pub fn new<T>(invrep: &[T]) -> Self
    where
        T: Borrow<inv::Repeat<Idx>>,
    {
        let (starts, revstart) = Index::index(invrep, |x| x.borrow().brange().start());
        let (ends, revend) = Index::index(invrep, |x| x.borrow().brange().end());

        let blocks = invrep
            .iter()
            .map(|x| {
                let blocks: Vec<_> = x.borrow().seqranges().cloned().collect();
                debug_assert!(blocks
                    .iter()
                    .tuple_windows()
                    .all(|(prv, nxt)| prv.end() <= nxt.start()));

                blocks
            })
            .collect();

        Self {
            starts,
            ends,
            revstart,
            revend,
            blocks,
        }
    }

    pub fn ends(&self) -> &[IndexAnchor<Idx>] {
        &self.ends
    }

    pub fn starts(&self) -> &[IndexAnchor<Idx>] {
        &self.starts
    }

    pub fn revmap(&self, rnaid: usize) -> (usize, usize) {
        (self.revstart[rnaid], self.revend[rnaid])
    }

    pub fn blocks(&self, rnaid: usize) -> &[Segment<Idx>] {
        &self.blocks[rnaid]
    }

    fn index<T: Borrow<inv::Repeat<Idx>>>(
        rnas: &[T],
        key: impl for<'b> Fn(&'b T) -> Idx,
    ) -> (Vec<IndexAnchor<Idx>>, Vec<usize>) {
        let mut argsort = (0..rnas.len()).collect::<Vec<_>>();
        argsort.sort_by_key(|x| key(&rnas[*x]));

        let mut index = Vec::with_capacity(rnas.len());
        let mut revmap = vec![0; rnas.len()];

        let mut curkey = key(&rnas[argsort[0]]);
        let mut cache = vec![argsort[0]];

        for rnaind in argsort.into_iter().skip(1) {
            if key(&rnas[rnaind]) != curkey {
                for ind in &cache {
                    revmap[*ind] = index.len();
                }
                index.push(IndexAnchor {
                    pos: curkey,
                    repeats: cache,
                });

                curkey = key(&rnas[rnaind]);
                cache = vec![rnaind];
            } else {
                cache.push(rnaind);
            }
        }
        for ind in &cache {
            revmap[*ind] = index.len();
        }
        index.push(IndexAnchor {
            pos: curkey,
            repeats: cache,
        });
        (index, revmap)
    }
}

pub mod bisect {
    use biobit_core_rs::num::PrimInt;

    use super::IndexAnchor;

    pub fn right<Idx: PrimInt>(
        data: &[IndexAnchor<Idx>],
        pos: Idx,
        mut lo: usize,
        mut hi: usize,
    ) -> usize {
        debug_assert!(lo <= hi && hi <= data.len());
        while lo < hi {
            let mid = (lo + hi) / 2;
            if pos < data[mid].pos {
                hi = mid
            } else {
                lo = mid + 1
            };
        }
        lo
    }

    pub fn left<Idx: PrimInt>(
        data: &[IndexAnchor<Idx>],
        pos: Idx,
        mut lo: usize,
        mut hi: usize,
    ) -> usize {
        debug_assert!(lo <= hi && hi <= data.len());
        while lo < hi {
            let mid = (lo + hi) / 2;
            if data[mid].pos < pos {
                lo = mid + 1
            } else {
                hi = mid
            };
        }
        lo
    }
}
