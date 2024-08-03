use derive_getters::{Dissolve, Getters};
use eyre::{eyre, Result};

use biobit_core_rs::loc::Segment;
use biobit_core_rs::num::{Float, PrimInt};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Dissolve, Getters)]
pub struct Peak<Idx: PrimInt, V> {
    segment: Segment<Idx>,
    signal: V,
    summit: Idx,
}

impl<Idx: PrimInt, V: Float> Peak<Idx, V> {
    pub fn new(start: Idx, end: Idx, signal: V, summit: Idx) -> Result<Self> {
        if summit > end || summit < start {
            return Err(eyre!(
                "Summit must be within the segment, got {:?} for [{:?}, {:?}]",
                summit,
                start,
                end
            ));
        }
        let segment = Segment::new(start, end)?;
        Ok(Self {
            segment,
            signal,
            summit,
        })
    }

    pub fn shift(&mut self, shift: Idx) {
        self.segment.shift(shift);
        self.summit = self.summit + shift;
    }
}
