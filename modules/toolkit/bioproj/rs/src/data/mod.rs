//! Stored data artifacts and their acquisition-level dataset boundaries.

pub mod asset;
pub mod dataset;
mod validate;

pub use asset::{Asset, AssetId, AssetIdRef};
pub use dataset::{Dataset, DatasetId, DatasetIdRef};

use crate::UntypedId;
use crate::provenance::Provenance;
use crate::validation;
use eyre::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

/// The resolved data domain for a released project.
///
/// Assets describe individual stored files. Datasets arrange those files into
/// complete representations of acquisitions. Construction resolves and
/// validates their shallow references against [`Provenance`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Data {
    assets: BTreeMap<UntypedId, Asset>,
    datasets: BTreeMap<UntypedId, Dataset>,
}

impl Data {
    /// Constructs and validates a complete data domain.
    pub fn new(
        provenance: &Provenance,
        assets: impl IntoIterator<Item = Asset>,
        datasets: impl IntoIterator<Item = Dataset>,
    ) -> Result<Self> {
        let assets: Vec<_> = assets.into_iter().collect();
        let datasets: Vec<_> = datasets.into_iter().collect();

        validation::unique_ids(
            "provenance and data",
            provenance
                .ids()
                .chain(assets.iter().map(|asset| asset.id().as_untyped()))
                .chain(datasets.iter().map(|dataset| dataset.id().as_untyped())),
        )?;

        let data = Self {
            assets: assets
                .into_iter()
                .map(|asset| (asset.id().as_untyped().clone(), asset))
                .collect(),
            datasets: datasets
                .into_iter()
                .map(|dataset| (dataset.id().as_untyped().clone(), dataset))
                .collect(),
        };
        validate::validate(provenance, &data.assets, &data.datasets)?;
        Ok(data)
    }

    /// Returns assets keyed by their globally unique untyped IDs.
    pub fn assets(&self) -> &BTreeMap<UntypedId, Asset> {
        &self.assets
    }

    /// Finds an asset by its globally unique untyped ID.
    pub fn asset(&self, id: &UntypedId) -> Option<&Asset> {
        self.assets.get(id)
    }

    /// Returns datasets keyed by their globally unique untyped IDs.
    pub fn datasets(&self) -> &BTreeMap<UntypedId, Dataset> {
        &self.datasets
    }

    /// Finds a dataset by its globally unique untyped ID.
    pub fn dataset(&self, id: &UntypedId) -> Option<&Dataset> {
        self.datasets.get(id)
    }

    /// Deserializes and validates data using its parent provenance graph.
    pub fn deserialize_with_provenance<'de, D>(
        provenance: &Provenance,
        deserializer: D,
    ) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = DeserializedData::deserialize(deserializer)?;
        Self::new(provenance, data.assets, data.datasets).map_err(serde::de::Error::custom)
    }

    /// Validates this data domain against its parent provenance graph.
    pub(crate) fn validate_against(&self, provenance: &Provenance) -> Result<()> {
        validation::unique_ids("provenance and data", provenance.ids().chain(self.ids()))?;
        validate::validate(provenance, &self.assets, &self.datasets)
    }

    /// Iterates over IDs occupied by this data domain.
    pub(crate) fn ids(&self) -> impl Iterator<Item = &UntypedId> {
        self.assets.keys().chain(self.datasets.keys())
    }
}

#[derive(Serialize)]
struct SerializedData<'a> {
    assets: Vec<&'a Asset>,
    datasets: Vec<&'a Dataset>,
}

impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedData {
            assets: self.assets.values().collect(),
            datasets: self.datasets.values().collect(),
        }
        .serialize(serializer)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeserializedData {
    assets: Vec<Asset>,
    datasets: Vec<Dataset>,
}

#[cfg(test)]
mod tests;
