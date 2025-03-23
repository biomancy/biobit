use bitcode::{Decode, Encode};
use derive_getters::Getters;
use derive_more::Into;
use eyre::{bail, Result};
use std::num::NonZeroUsize;

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Getters, Into)]
pub struct Deflate {
    level: u8,
}

impl Deflate {
    pub const NONE: Deflate = Deflate { level: 0 };
    pub const FAST: Deflate = Deflate { level: 1 };
    pub const DEFAULT: Deflate = Deflate { level: 6 };
    pub const BEST: Deflate = Deflate { level: 9 };

    pub fn new(level: u8) -> Result<Self> {
        if level > 9 {
            bail!("Invalid DEFLATE compression level: {}", level);
        }
        Ok(Self { level })
    }
}

impl Default for Deflate {
    fn default() -> Self {
        Deflate::DEFAULT
    }
}

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Getters, Into)]
pub struct Bgzf {
    deflate: Deflate,
    threads: NonZeroUsize,
}

impl Bgzf {
    pub const DEFAULT: Bgzf = Bgzf {
        deflate: Deflate::DEFAULT,
        threads: NonZeroUsize::new(1).unwrap(),
    };

    pub fn new(deflate: Deflate, threads: NonZeroUsize) -> Result<Self> {
        Ok(Self { deflate, threads })
    }
}

impl Default for Bgzf {
    fn default() -> Self {
        Bgzf::DEFAULT
    }
}
