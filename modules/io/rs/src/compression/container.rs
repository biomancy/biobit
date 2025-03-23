use bitcode::{Decode, Encode};
use eyre::{bail, Error, Result};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Container {
    None,
    Gzip,
    Bgzf,
}

impl FromStr for Container {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Container::None),
            "gzip" => Ok(Container::Gzip),
            "bgzf" => Ok(Container::Bgzf),
            _ => bail!("Unknown compression container: {}", s),
        }
    }
}

impl Display for Container {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Container::None => write!(f, "none"),
            Container::Gzip => write!(f, "gzip"),
            Container::Bgzf => write!(f, "bgzf"),
        }
    }
}
