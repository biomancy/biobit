use derive_getters::Dissolve;
use derive_more::Constructor;

use biobit_core_rs::num::{Float, PrimInt};

pub use super::pcalling::Config as PCalling;

#[derive(Clone, PartialEq, Debug, Constructor, Dissolve)]
pub struct Config<Idx: PrimInt, Cnts: Float> {
    // Sensitivity
    pub sensitivity: Cnts,
    // Control/Signal parameters
    pub control_scaling: Cnts,
    pub signal_scaling: Cnts,
    pub control_baseline: Cnts,
    pub min_raw_signal: Cnts,
    // Peak calling parameters
    pub pcalling: PCalling<Idx, Cnts>,
}

impl<Idx: PrimInt, Cnts: Float> Default for Config<Idx, Cnts> {
    fn default() -> Self {
        Config {
            sensitivity: Cnts::min_positive_value(),
            control_baseline: Cnts::zero(),
            control_scaling: Cnts::one(),
            signal_scaling: Cnts::one(),
            min_raw_signal: Cnts::zero(),
            pcalling: PCalling::new(Idx::zero(), Idx::zero(), Cnts::zero()),
        }
    }
}
