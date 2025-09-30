use super::utils;
use derive_getters::{Dissolve, Getters};
use std::ops::Range;
use biobit_core_rs::loc::Interval;

#[derive(Clone, PartialEq, Debug, Getters, Dissolve)]
pub struct Payload {
    partition: Interval<u64>,
    to_process: Vec<Range<u64>>,
}

impl Payload {
    pub fn from_intervals(
        intervals: Vec<Interval<u64>>,
        binsize: u64,
        exclude: Option<Vec<Interval<u64>>>,
    ) -> Vec<Payload> {
        assert!(binsize > 0, "Binsize must be > 0");
        assert!(exclude.is_none(), "exclude must be none");

        // Bin and transform to the payload
        let intervals = utils::split(intervals, binsize);
        utils::bin(intervals, binsize)
            .into_iter()
            .map(|x| Payload {
                partition: x.bin,
                to_process: x.items.into_iter().map(|x| x.into()).collect(),
            })
            .collect()
    }
}
