use crate::data::Data;
use crate::data::asset::fastq::FastqId;
use crate::primitives::define_entity_id;
use crate::provenance::acquisition::illumina::PairedEndSequencingId;
use crate::provenance::{AcquisitionIdRef, Provenance};
use crate::{Error, Meta, NonEmpty, OrderedSet, Result, Text, UntypedId};
use std::collections::BTreeSet;

define_entity_id!(
    PairedId,
    "The identifier of a [`crate::data::dataset::fastq::Paired`] dataset."
);

/// An acquisition compatible with an ordered-pair FASTQ dataset.
///
/// This dataset-owned contract can grow to include other paired FASTQ
/// producers without coupling [`crate::data::Asset`] to acquisition semantics.
#[derive(Clone, Debug, Eq, PartialEq, derive_more::From)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub enum PairedInput {
    /// Paired-end Illumina sequencing.
    IlluminaPairedEndSequencing(PairedEndSequencingId),
}

impl PairedInput {
    /// Returns the concrete borrowed acquisition identifier.
    pub fn id(&self) -> AcquisitionIdRef<'_> {
        match self {
            Self::IlluminaPairedEndSequencing(id) => id.into(),
        }
    }

    pub(crate) fn validate_against(
        &self,
        dataset_id: &UntypedId,
        provenance: &Provenance,
    ) -> Result<()> {
        match self {
            Self::IlluminaPairedEndSequencing(id) => {
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

/// One ordered read-one/read-two correspondence.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct Pair(FastqId, FastqId);

impl Pair {
    /// Creates an ordered pair of distinct FASTQ assets.
    pub fn new(read1: FastqId, read2: FastqId) -> Result<Self> {
        if read1 == read2 {
            return Err(Error::invalid(
                "PairedFastq::pair",
                "must use distinct Assets",
            ));
        }
        Ok(Self(read1, read2))
    }

    /// Returns the read-one FASTQ asset ID.
    pub fn read1(&self) -> &FastqId {
        &self.0
    }

    /// Returns the read-two FASTQ asset ID.
    pub fn read2(&self) -> &FastqId {
        &self.1
    }
}

/// A complete representation made from ordered FASTQ pairs.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct Paired {
    id: PairedId,
    acquisition: PairedInput,
    pairs: NonEmpty<OrderedSet<Pair>>,
    meta: Meta,
    description: Option<Text>,
}

impl Paired {
    /// Creates a paired FASTQ dataset.
    pub fn new(
        id: PairedId,
        acquisition: impl Into<PairedInput>,
        pairs: NonEmpty<OrderedSet<Pair>>,
        meta: Meta,
        description: Option<Text>,
    ) -> Result<Self> {
        validate_unique_assets(&pairs)?;
        Ok(Self {
            id,
            acquisition: acquisition.into(),
            pairs,
            meta,
            description,
        })
    }

    /// Returns this dataset's identifier.
    pub fn id(&self) -> &PairedId {
        &self.id
    }

    /// Returns the acquisition represented by this dataset.
    pub fn acquisition(&self) -> &PairedInput {
        &self.acquisition
    }

    pub(crate) fn validate_against(&self, data: &Data) -> Result<()> {
        for pair in &self.pairs {
            for asset_id in [pair.read1(), pair.read2()] {
                match data.get(asset_id) {
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(error),
                    None => {
                        return Err(Error::missing(
                            "Dataset",
                            self.id.as_untyped(),
                            "pairs",
                            "Asset",
                            asset_id.as_untyped(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Returns the ordered FASTQ pairs.
    pub fn pairs(&self) -> &NonEmpty<OrderedSet<Pair>> {
        &self.pairs
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

fn validate_unique_assets(pairs: &NonEmpty<OrderedSet<Pair>>) -> Result<()> {
    let mut assets = BTreeSet::new();
    for pair in pairs {
        for id in [pair.read1(), pair.read2()] {
            if !assets.insert(id) {
                return Err(Error::invalid(
                    "PairedFastq::pairs",
                    format!("FASTQ Asset '{id}' appears more than once"),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Pair, Paired, PairedId};
    use crate::NonEmpty;
    use crate::data::asset::fastq::FastqId;
    use crate::provenance::acquisition::illumina::PairedEndSequencingId;

    fn pair(read1: &str, read2: &str) -> Pair {
        Pair::new(FastqId::new(read1).unwrap(), FastqId::new(read2).unwrap()).unwrap()
    }

    #[test]
    fn rejects_an_asset_in_more_than_one_pair() {
        let error = Paired::new(
            PairedId::new("DATA1").unwrap(),
            PairedEndSequencingId::new("ACQ1").unwrap(),
            NonEmpty::try_from_iter([pair("A", "B"), pair("A", "C")]).unwrap(),
            Default::default(),
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("appears more than once"));
    }
}
