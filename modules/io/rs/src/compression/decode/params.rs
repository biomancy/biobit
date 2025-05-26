#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::Getters;
use derive_more::Into;
use eyre::Result;
use std::num::NonZeroUsize;

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Getters, Into)]
pub struct Bgzf {
    threads: NonZeroUsize,
}

impl Bgzf {
    pub const DEFAULT: Bgzf = Bgzf {
        threads: NonZeroUsize::new(1).unwrap(),
    };

    pub fn new(threads: NonZeroUsize) -> Result<Self> {
        Ok(Self { threads })
    }
}

impl Default for Bgzf {
    fn default() -> Self {
        Bgzf::DEFAULT
    }
}
