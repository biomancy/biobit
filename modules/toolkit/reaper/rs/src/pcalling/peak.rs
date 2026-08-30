use derive_getters::{Dissolve, Getters};
use eyre::{Result, eyre};

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
        if summit >= end || summit < start {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summit_must_be_inside_the_half_open_interval() {
        assert!(Peak::new(10usize, 20, 1.0f64, 20).is_err());
        assert!(Peak::new(10usize, 20, 1.0f64, 9).is_err());
        assert!(Peak::new(10usize, 20, 1.0f64, 10).is_ok());
        assert!(Peak::new(10usize, 20, 1.0f64, 19).is_ok());
    }

    #[test]
    fn shifting_moves_the_interval_and_summit_together() -> Result<()> {
        let mut peak = Peak::new(1usize, 3, 2.0f64, 2)?;
        peak.shift(100);
        assert_eq!(*peak.interval(), Interval::new(101, 103)?);
        assert_eq!(*peak.summit(), 102);
        Ok(())
    }
}
