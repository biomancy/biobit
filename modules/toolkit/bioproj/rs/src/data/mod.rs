//! Stored data artifacts and their acquisition-level dataset boundaries.

pub mod asset;
pub mod dataset;
mod root;

pub use asset::{Asset, AssetId, AssetIdRef, AssetKind};
pub use dataset::{Dataset, DatasetId, DatasetIdRef, DatasetKind};
pub use root::Data;
