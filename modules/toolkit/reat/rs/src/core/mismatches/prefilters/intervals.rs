use std::ops::Range;

use super::SitesRetainer;
use biobit_collections_rs::interval_tree::Bits;
use biobit_core_rs::loc::Interval;

#[derive(Clone)]
pub struct RetainSitesFromIntervals {
    _index: Bits<u64, ()>,
}

impl RetainSitesFromIntervals {
    pub fn new(include: Vec<Interval<u64>>) -> Self {
        let index = Bits::new(include.into_iter().map(|x| (x, ())));
        Self { _index: index }
    }
}

impl SitesRetainer for RetainSitesFromIntervals {
    #[inline]
    fn retained(&self, _contig: &str, _range: Range<u64>) -> Vec<Range<u64>> {
        // match self.index.get(contig) {
        //     None => vec![],
        //     Some(map) => {
        //         let mut results = Vec::new();
        //         for hit in map.find(range) {
        //             results.push(hit.interval().start..hit.interval().end)
        //         }
        //         results.sort_by_key(|x| x.start);
        //         results
        //     }
        // }
        todo!()
    }
}
