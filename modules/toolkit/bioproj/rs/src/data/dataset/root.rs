use super::fastq;
use crate::data::Data;
use crate::primitives::define_entity_family;
use crate::provenance::{AcquisitionIdRef, Provenance};
use crate::{Error, Result, UntypedId};
use std::collections::{BTreeMap, BTreeSet};

define_entity_family! {
    /// A complete stored form of one acquisition.
    ///
    /// All datasets for an acquisition declare the same complete released
    /// collection and interpretation, using different file layouts or encodings.
    /// Reusing assets across equivalent representations is allowed. Selected
    /// subsets require a future explicit selection model.
    pub family Dataset {
        kind: DatasetKind,
        id: DatasetId,
        id_ref: DatasetIdRef,
        kind_doc: "The concrete type of a dataset or dataset identifier.",
        variants: {
            /// One or more independent FASTQ files.
            Fastq(fastq::Single, fastq::SingleId),
            /// One or more ordered read-one/read-two FASTQ pairs.
            PairedFastq(fastq::Paired, fastq::PairedId),
        }
    }
}

impl Dataset {
    /// Returns the acquisition represented by this dataset.
    pub fn acquisition(&self) -> AcquisitionIdRef<'_> {
        match self {
            Self::Fastq(dataset) => dataset.acquisition().id(),
            Self::PairedFastq(dataset) => dataset.acquisition().id(),
        }
    }

    /// Checks Asset relationships and coverage within Data.
    ///
    /// 1. Each Dataset must refer to existing Assets.
    /// 2. An Asset may occur in multiple Datasets only when they represent the
    ///    same Acquisition.
    /// 3. For one Acquisition, Datasets of the same kind must not have Asset sets
    ///    that are strict subsets of each other.
    /// 4. Every Asset must be used by at least one Dataset.
    ///
    /// These checks compare IDs; authors establish whether different Datasets
    /// actually contain equivalent observations.
    pub(crate) fn validate(data: &Data) -> Result<()> {
        let mut assets: BTreeMap<UntypedId, AcquisitionIdRef<'_>> = BTreeMap::new();
        let mut representations: BTreeMap<UntypedId, Vec<(&Dataset, BTreeSet<UntypedId>)>> =
            BTreeMap::new();

        for dataset in data.datasets() {
            // Check Asset references, then collect this representation's IDs.
            let members: BTreeSet<_> = match dataset {
                Self::Fastq(single) => {
                    single.validate_against(data)?;
                    single
                        .assets()
                        .iter()
                        .map(|id| id.as_untyped().clone())
                        .collect()
                }
                Self::PairedFastq(paired) => {
                    paired.validate_against(data)?;
                    paired
                        .pairs()
                        .iter()
                        .flat_map(|pair| [pair.read1(), pair.read2()])
                        .map(|id| id.as_untyped().clone())
                        .collect()
                }
            };

            // An Asset may be shared only by representations of one Acquisition.
            let acquisition = dataset.acquisition();
            for asset_id in &members {
                match assets.get(asset_id) {
                    Some(previous) if *previous != acquisition => {
                        return Err(Error::AssetOwnership {
                            asset: asset_id.clone(),
                            first_acquisition: previous.as_untyped().clone(),
                            second_acquisition: acquisition.as_untyped().clone(),
                        });
                    }
                    Some(_) => {}
                    None => {
                        assets.insert(asset_id.clone(), acquisition);
                    }
                }
            }

            // Detect strict Asset-set subsets within one Dataset kind.
            let prior = representations
                .entry(acquisition.as_untyped().clone())
                .or_default();
            for (other, other_members) in prior.iter() {
                if dataset.kind() != other.kind() || members.len() == other_members.len() {
                    continue;
                }
                let (subset, superset) = if members.is_subset(other_members) {
                    (dataset, *other)
                } else if other_members.is_subset(&members) {
                    (*other, dataset)
                } else {
                    continue;
                };
                return Err(Error::DatasetSubset {
                    acquisition: dataset.acquisition().as_untyped().clone(),
                    subset: subset.id().as_untyped().clone(),
                    superset: superset.id().as_untyped().clone(),
                });
            }
            prior.push((dataset, members));
        }

        // Every Asset must belong to at least one Dataset.
        for asset in data.assets() {
            let asset_id = asset.id().as_untyped();
            if !assets.contains_key(asset_id) {
                return Err(Error::UnusedAsset {
                    id: asset_id.clone(),
                });
            }
        }
        Ok(())
    }

    /// Checks Acquisition references, compatibility, and coverage against Provenance.
    pub(crate) fn validate_against(provenance: &Provenance, data: &Data) -> Result<()> {
        let mut represented = BTreeSet::new();
        for dataset in data.datasets() {
            match dataset {
                Self::Fastq(single) => single
                    .acquisition()
                    .validate_against(single.id().as_untyped(), provenance)?,
                Self::PairedFastq(paired) => paired
                    .acquisition()
                    .validate_against(paired.id().as_untyped(), provenance)?,
            }
            represented.insert(dataset.acquisition().as_untyped().clone());
        }

        for acquisition in provenance.acquisitions() {
            let id = acquisition.id().as_untyped();
            if !represented.contains(id) {
                return Err(Error::UnrepresentedAcquisition { id: id.clone() });
            }
        }
        Ok(())
    }
}
