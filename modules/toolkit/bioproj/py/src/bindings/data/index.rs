use biobit_bioproj_rs as rs;
use pyo3::prelude::*;

use super::asset::{AssetIdInput, AssetInput};
use super::dataset::{DatasetIdInput, DatasetInput};
use crate::utils::lookup::define_family_record_index;

define_family_record_index!(
    PyAssetIndex(rs::data::Data) as "AssetIndex",
    record: AssetInput,
    id: AssetIdInput as "AssetId",
    id_ref: rs::data::AssetIdRef<'_>,
    rust: rs::data::Asset,
    lookup: asset,
    iter: assets
);

define_family_record_index!(
    PyDatasetIndex(rs::data::Data) as "DatasetIndex",
    record: DatasetInput,
    id: DatasetIdInput as "DatasetId",
    id_ref: rs::data::DatasetIdRef<'_>,
    rust: rs::data::Dataset,
    lookup: dataset,
    iter: datasets
);
