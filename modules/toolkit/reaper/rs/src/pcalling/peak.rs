use derive_getters::{Dissolve, Getters};
use eyre::{eyre, Result};

use biobit_core_rs::loc::Interval;
use biobit_core_rs::num::{Float, PrimInt};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Dissolve, Getters)]
pub struct Peak<Idx: PrimInt, V> {
    interval: Interval<Idx>,
    signal: V,
    summit: Idx,
}

impl<Idx: PrimInt, V: Float> Peak<Idx, V> {
    pub fn new(start: Idx, end: Idx, signal: V, summit: Idx) -> Result<Self> {
        if summit > end || summit < start {
            return Err(eyre!(
                "Summit must be within the interval, got {:?} for [{:?}, {:?}]",
                summit,
                start,
                end
            ));
        }
        let interval = Interval::new(start, end)?;
        Ok(Self {
            interval,
            signal,
            summit,
        })
    }

    pub fn shift(&mut self, shift: Idx) {
        self.interval.shift(shift);
        self.summit = self.summit + shift;
    }
}
