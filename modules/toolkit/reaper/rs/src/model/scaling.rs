use biobit_core_rs::num::Float;
#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::{Dissolve, Getters};
use eyre::{Result, eyre};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Eq, Debug, Getters, Dissolve)]
pub struct Scaling<Cnts: Float> {
    signal: Cnts,
    control: Cnts,
}

impl<Cnts: Float> Scaling<Cnts> {
    pub fn new(signal: Cnts, control: Cnts) -> Result<Self> {
        if !signal.is_finite() || signal <= Cnts::zero() {
            return Err(eyre!("Signal scaling must be finite and greater than zero"));
        }
        if !control.is_finite() || control <= Cnts::zero() {
            return Err(eyre!(
                "Control scaling must be finite and greater than zero"
            ));
        }
        Ok(Self { signal, control })
    }
}

impl<Cnts: Float> Default for Scaling<Cnts> {
    fn default() -> Self {
        Self {
            signal: Cnts::one(),
            control: Cnts::one(),
        }
    }
}
