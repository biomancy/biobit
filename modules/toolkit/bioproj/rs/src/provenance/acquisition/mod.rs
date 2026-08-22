//! Logical data acquisitions grouped by measurement family.

pub mod illumina;

use crate::{Meta, NonEmpty, UntypedId};
use eyre::Result;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeMap;

use super::Library;

/// The owned identifier of any concrete acquisition.
///
/// Concrete IDs remain distinct at compatibility boundaries. This closed
/// union is used only by heterogeneous relationships such as design units and
/// datasets.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum AcquisitionId {
    /// A single-end Illumina sequencing acquisition.
    IlluminaSingleEndSequencing(illumina::SingleEndSequencingId),
    /// A paired-end Illumina sequencing acquisition.
    IlluminaPairedEndSequencing(illumina::PairedEndSequencingId),
}

impl AcquisitionId {
    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(&self) -> &UntypedId {
        match self {
            Self::IlluminaSingleEndSequencing(id) => id.as_untyped(),
            Self::IlluminaPairedEndSequencing(id) => id.as_untyped(),
        }
    }
}

impl Serialize for AcquisitionId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_untyped().serialize(serializer)
    }
}

impl From<illumina::SingleEndSequencingId> for AcquisitionId {
    fn from(id: illumina::SingleEndSequencingId) -> Self {
        Self::IlluminaSingleEndSequencing(id)
    }
}

impl From<illumina::PairedEndSequencingId> for AcquisitionId {
    fn from(id: illumina::PairedEndSequencingId) -> Self {
        Self::IlluminaPairedEndSequencing(id)
    }
}

/// A borrowed identifier of any concrete acquisition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcquisitionIdRef<'a> {
    /// A single-end Illumina sequencing acquisition.
    IlluminaSingleEndSequencing(&'a illumina::SingleEndSequencingId),
    /// A paired-end Illumina sequencing acquisition.
    IlluminaPairedEndSequencing(&'a illumina::PairedEndSequencingId),
}

impl<'a> AcquisitionIdRef<'a> {
    /// Returns the shared workspace-local identifier.
    pub fn as_untyped(self) -> &'a UntypedId {
        match self {
            Self::IlluminaSingleEndSequencing(id) => id.as_untyped(),
            Self::IlluminaPairedEndSequencing(id) => id.as_untyped(),
        }
    }

    /// Clones this borrowed identifier into its owned union.
    pub fn to_owned(self) -> AcquisitionId {
        match self {
            Self::IlluminaSingleEndSequencing(id) => {
                AcquisitionId::IlluminaSingleEndSequencing(id.clone())
            }
            Self::IlluminaPairedEndSequencing(id) => {
                AcquisitionId::IlluminaPairedEndSequencing(id.clone())
            }
        }
    }
}

/// A concrete logical acquisition.
///
/// This initial type intentionally combines two concepts: the reusable assay
/// specification and one execution of that assay. A future model may separate
/// them so repeated acquisitions can reference one shared method definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Acquisition {
    /// Single-end Illumina sequencing.
    IlluminaSingleEndSequencing(illumina::SingleEndSequencing),
    /// Paired-end Illumina sequencing.
    IlluminaPairedEndSequencing(illumina::PairedEndSequencing),
}

impl Acquisition {
    /// Returns this acquisition's concrete borrowed identifier.
    pub fn id(&self) -> AcquisitionIdRef<'_> {
        match self {
            Self::IlluminaSingleEndSequencing(acquisition) => {
                AcquisitionIdRef::IlluminaSingleEndSequencing(acquisition.id())
            }
            Self::IlluminaPairedEndSequencing(acquisition) => {
                AcquisitionIdRef::IlluminaPairedEndSequencing(acquisition.id())
            }
        }
    }

    /// Returns the untyped workspace-local identifier.
    pub fn untyped_id(&self) -> &UntypedId {
        self.id().as_untyped()
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        match self {
            Self::IlluminaSingleEndSequencing(acquisition) => acquisition.meta(),
            Self::IlluminaPairedEndSequencing(acquisition) => acquisition.meta(),
        }
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&NonEmpty<String>> {
        match self {
            Self::IlluminaSingleEndSequencing(acquisition) => acquisition.description(),
            Self::IlluminaPairedEndSequencing(acquisition) => acquisition.description(),
        }
    }

    /// Validates this acquisition's material compatibility contract.
    pub(crate) fn validate(&self, libraries: &BTreeMap<UntypedId, Library>) -> Result<()> {
        match self {
            Self::IlluminaSingleEndSequencing(acquisition) => acquisition.validate(libraries),
            Self::IlluminaPairedEndSequencing(acquisition) => acquisition.validate(libraries),
        }
    }
}

impl From<illumina::SingleEndSequencing> for Acquisition {
    fn from(acquisition: illumina::SingleEndSequencing) -> Self {
        Self::IlluminaSingleEndSequencing(acquisition)
    }
}

impl From<illumina::PairedEndSequencing> for Acquisition {
    fn from(acquisition: illumina::PairedEndSequencing) -> Self {
        Self::IlluminaPairedEndSequencing(acquisition)
    }
}
