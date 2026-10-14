use super::asset::fastq as asset_fastq;
use super::dataset::fastq as dataset_fastq;
use super::{Asset, AssetId, AssetIdRef, Dataset, DatasetId, DatasetIdRef};
use crate::lookup;
use crate::primitives::{OrderedMap, impl_checked_lookup, impl_variant_lookup};
use crate::provenance::Provenance;
use crate::{Lookup, Result, UntypedId};

/// Assets and their complete Dataset representations.
///
/// Construction checks relationships within Data. [`crate::Project`] checks
/// Acquisition relationships against [`crate::provenance::Provenance`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct Data {
    assets: OrderedMap<UntypedId, Asset>,
    datasets: OrderedMap<UntypedId, Dataset>,
}

impl Data {
    /// Constructs Data and checks its Asset and Dataset relationships.
    pub fn new(
        assets: impl IntoIterator<Item = Asset>,
        datasets: impl IntoIterator<Item = Dataset>,
    ) -> Result<Self> {
        let data = Self {
            assets: OrderedMap::try_from_iter(
                "data.assets",
                assets
                    .into_iter()
                    .map(|asset| (asset.id().as_untyped().clone(), asset)),
            )?,
            datasets: OrderedMap::try_from_iter(
                "data.datasets",
                datasets
                    .into_iter()
                    .map(|dataset| (dataset.id().as_untyped().clone(), dataset)),
            )?,
        };
        Dataset::validate(&data)?;
        Ok(data)
    }

    /// Iterates over all assets.
    pub fn assets(&self) -> impl ExactSizeIterator<Item = &Asset> + '_ {
        self.assets.values()
    }

    /// Iterates over all datasets.
    pub fn datasets(&self) -> impl ExactSizeIterator<Item = &Dataset> + '_ {
        self.datasets.values()
    }

    /// Performs a lookup whose result is determined by its typed ID.
    pub fn get<'a, K: ?Sized>(
        &'a self,
        key: &K,
    ) -> lookup::OptResult<&'a <Self as Lookup<K>>::Target, <Self as Lookup<K>>::Err>
    where
        Self: Lookup<K>,
    {
        <Self as Lookup<K>>::lookup(self, key)
    }

    /// Finds an asset by its untyped ID.
    pub fn asset(&self, id: &UntypedId) -> Option<&Asset> {
        self.assets.get(id)
    }

    /// Finds a dataset by its untyped ID.
    pub fn dataset(&self, id: &UntypedId) -> Option<&Dataset> {
        self.datasets.get(id)
    }

    /// Checks Dataset-to-Acquisition relationships against Provenance.
    pub(crate) fn validate_against(&self, provenance: &Provenance) -> Result<()> {
        Dataset::validate_against(provenance, self)
    }
}

impl_checked_lookup!(Data, AssetId, Asset, assets);
impl_checked_lookup!(Data, AssetIdRef<'_>, Asset, assets);
impl_checked_lookup!(Data, DatasetId, Dataset, datasets);
impl_checked_lookup!(Data, DatasetIdRef<'_>, Dataset, datasets);

impl_variant_lookup!(
    Data,
    asset_fastq::FastqId,
    asset_fastq::Fastq,
    assets,
    Asset::Fastq
);
impl_variant_lookup!(
    Data,
    dataset_fastq::SingleId,
    dataset_fastq::Single,
    datasets,
    Dataset::Fastq
);
impl_variant_lookup!(
    Data,
    dataset_fastq::PairedId,
    dataset_fastq::Paired,
    datasets,
    Dataset::PairedFastq
);
