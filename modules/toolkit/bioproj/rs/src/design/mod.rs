//! Experimental-design topology over a provenance graph.

pub mod group_contrast;
pub mod matched_pairs;
mod root;
pub mod unit;
pub mod unit_contrast;
pub mod unit_set;

pub use group_contrast::{GroupContrast, GroupContrastId};
pub use matched_pairs::{MatchedPair, MatchedPairs, MatchedPairsId};
pub use root::{Design, DesignId, DesignIdRef, DesignKind, Designs};
pub use unit::{Unit, UnitId};
pub use unit_contrast::{UnitContrast, UnitContrastId};
pub use unit_set::{UnitSet, UnitSetId};
