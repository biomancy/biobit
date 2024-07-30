use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;

use biobit_collections_rs::rle_vec::{Identical, RleVec};
use biobit_core_rs::num::{Float, PrimInt, PrimUInt};

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve)]
pub struct Config<Idx, V> {
    pub min_length: Idx,
    pub merge_within: Idx,
    pub cutoff: V,
}

#[derive(Clone, PartialEq, Debug, Default, Constructor, Dissolve, Getters)]
pub struct Peak<Idx, V> {
    start: Idx,
    end: Idx,
    signal: V,
    summit: Idx,
}

pub fn run<Idx: PrimInt, V: Float, L: PrimUInt, I: Identical<V>>(
    rle: &RleVec<V, L, I>,
    cfg: &Config<Idx, V>,
) -> Vec<Peak<Idx, V>> {
    let mut cursor = Idx::zero();
    let div = Idx::from(2).unwrap();

    let mut pieces = rle.runs().filter_map(|(val, length)| {
        let end = cursor + Idx::from(*length).unwrap();
        let result = if *val >= cfg.cutoff {
            Some(Peak::new(cursor, end, val.clone(), (cursor + end) / div))
        } else {
            None
        };
        cursor = end;
        result
    });

    let mut result = Vec::new();
    let mut current = match pieces.next() {
        Some(x) => x,
        None => return Vec::new(),
    };

    for next in pieces {
        if next.start - current.end > cfg.merge_within {
            result.push(current);
            current = next;
        } else {
            // Merge the two peaks
            current.end = next.end;

            if next.signal > current.signal {
                current.signal = next.signal;
                current.summit = next.summit;
            }
        }
    }

    result.push(current);
    result
}
