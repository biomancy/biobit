use crate::data::Data;
use crate::data::asset::fastq::FastqId;
use crate::primitives::define_entity_id;
use crate::provenance::acquisition::illumina::SingleEndSequencingId;
use crate::provenance::{AcquisitionIdRef, Provenance};
use crate::{Error, Meta, NonEmpty, OrderedSet, Result, Text, UntypedId};

define_entity_id!(
    SingleId,
    "The identifier of a [`crate::data::dataset::fastq::Single`] dataset."
);

/// An acquisition compatible with an independent-file FASTQ dataset.
///
/// This dataset-owned contract can grow to include other single-file FASTQ
/// producers without coupling [`crate::data::Asset`] to acquisition semantics.
#[derive(Clone, Debug, Eq, PartialEq, derive_more::From)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum SingleInput {
    /// Single-end Illumina sequencing.
    IlluminaSingleEndSequencing(SingleEndSequencingId),
}

impl SingleInput {
    /// Returns the concrete borrowed acquisition identifier.
    pub fn id(&self) -> AcquisitionIdRef<'_> {
        match self {
            Self::IlluminaSingleEndSequencing(id) => id.into(),
        }
    }

    pub(crate) fn validate_against(
        &self,
        dataset_id: &UntypedId,
        provenance: &Provenance,
    ) -> Result<()> {
        match self {
            Self::IlluminaSingleEndSequencing(id) => {
                let key: AcquisitionIdRef<'_> = id.into();
                match provenance.get(&key) {
                    Some(Ok(_)) => Ok(()),
                    Some(Err(error)) => Err(error),
                    None => Err(Error::missing(
                        "Dataset",
                        dataset_id,
                        "acquisition",
                        "Acquisition",
                        id.as_untyped(),
                    )),
                }
            }
        }
    }
}

/// A complete representation made from independent FASTQ files.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct Single {
    id: SingleId,
    acquisition: SingleInput,
    assets: NonEmpty<OrderedSet<FastqId>>,
    meta: Meta,
    description: Option<Text>,
}

impl Single {
    /// Creates a FASTQ dataset.
    pub fn new(
        id: SingleId,
        acquisition: impl Into<SingleInput>,
        assets: NonEmpty<OrderedSet<FastqId>>,
        meta: Meta,
        description: Option<Text>,
    ) -> Self {
        Self {
            id,
            acquisition: acquisition.into(),
            assets,
            meta,
            description,
        }
    }

    /// Returns this dataset's identifier.
    pub fn id(&self) -> &SingleId {
        &self.id
    }

    /// Returns the acquisition represented by this dataset.
    pub fn acquisition(&self) -> &SingleInput {
        &self.acquisition
    }

    pub(crate) fn validate_against(&self, data: &Data) -> Result<()> {
        for asset_id in &self.assets {
            match data.get(asset_id) {
                Some(Ok(_)) => {}
                Some(Err(error)) => return Err(error),
                None => {
                    return Err(Error::missing(
                        "Dataset",
                        self.id.as_untyped(),
                        "assets",
                        "Asset",
                        asset_id.as_untyped(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Returns the complete set of FASTQ assets.
    pub fn assets(&self) -> &NonEmpty<OrderedSet<FastqId>> {
        &self.assets
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&Text> {
        self.description.as_ref()
    }
}
