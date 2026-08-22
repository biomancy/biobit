use super::{Asset, Dataset};
use crate::UntypedId;
use crate::provenance::{AcquisitionId, AcquisitionIdRef, Provenance};
use eyre::{Result, bail};
use std::collections::{BTreeMap, BTreeSet};

/// Validates the relationships within a complete data domain.
pub(crate) fn validate(
    provenance: &Provenance,
    assets: &BTreeMap<UntypedId, Asset>,
    datasets: &BTreeMap<UntypedId, Dataset>,
) -> Result<()> {
    let mut represented_acquisitions = BTreeSet::new();
    let mut represented_assets: BTreeMap<UntypedId, AcquisitionId> = BTreeMap::new();

    for dataset in datasets.values() {
        validate_acquisition(dataset, provenance)?;
        represented_acquisitions.insert(dataset.acquisition().as_untyped().clone());

        match dataset {
            Dataset::Fastq(dataset) => {
                for asset_id in dataset.assets() {
                    validate_fastq_asset(asset_id.as_untyped(), assets)?;
                    record_asset_acquisition(
                        asset_id.as_untyped(),
                        dataset.acquisition().id(),
                        &mut represented_assets,
                    )?;
                }
            }
            Dataset::PairedFastq(dataset) => {
                for pair in dataset.pairs() {
                    for asset_id in [pair.read1(), pair.read2()] {
                        validate_fastq_asset(asset_id.as_untyped(), assets)?;
                        record_asset_acquisition(
                            asset_id.as_untyped(),
                            dataset.acquisition().id(),
                            &mut represented_assets,
                        )?;
                    }
                }
            }
        }
    }

    for acquisition_id in provenance.acquisitions().keys() {
        if !represented_acquisitions.contains(acquisition_id) {
            bail!("Acquisition '{acquisition_id}' has no Dataset");
        }
    }
    for asset_id in assets.keys() {
        if !represented_assets.contains_key(asset_id) {
            bail!("Asset '{asset_id}' does not belong to any Dataset");
        }
    }
    Ok(())
}

fn validate_acquisition(dataset: &Dataset, provenance: &Provenance) -> Result<()> {
    match dataset {
        Dataset::Fastq(dataset) => dataset.validate(provenance),
        Dataset::PairedFastq(dataset) => dataset.validate(provenance),
    }
}

fn validate_fastq_asset(id: &UntypedId, assets: &BTreeMap<UntypedId, Asset>) -> Result<()> {
    match assets.get(id) {
        Some(Asset::Fastq(_)) => Ok(()),
        None => bail!("Dataset references unknown FASTQ Asset '{id}'"),
    }
}

fn record_asset_acquisition(
    asset_id: &UntypedId,
    acquisition: AcquisitionIdRef<'_>,
    represented_assets: &mut BTreeMap<UntypedId, AcquisitionId>,
) -> Result<()> {
    let acquisition = acquisition.to_owned();
    match represented_assets.get(asset_id) {
        Some(previous) if previous != &acquisition => bail!(
            "Asset '{asset_id}' belongs to Datasets for different Acquisitions '{}' and '{}'",
            previous.as_untyped(),
            acquisition.as_untyped()
        ),
        Some(_) => Ok(()),
        None => {
            represented_assets.insert(asset_id.clone(), acquisition);
            Ok(())
        }
    }
}
