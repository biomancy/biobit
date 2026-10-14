use crate::data::Data;
use crate::primitives::define_entity_id;
use crate::provenance::Provenance;
use crate::{Designs, Meta, Result, Text};

define_entity_id!(ProjectId, "The identifier of a [`crate::Project`].");

/// A complete, immutable released biological-data project.
///
/// `Project` owns the released provenance, data, and design domains.
/// Storage adapters and schema versions belong to [`crate::io`]. A Project ID
/// denotes one immutable released description;
/// authors must assign a new ID when changing that description.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct Project {
    id: ProjectId,
    provenance: Provenance,
    data: Data,
    designs: Designs,
    meta: Meta,
    description: Option<Text>,
}

impl Project {
    /// Constructs and validates a complete released project.
    pub fn new(
        id: ProjectId,
        provenance: Provenance,
        data: Data,
        designs: Designs,
        meta: Meta,
        description: Option<Text>,
    ) -> Result<Self> {
        let project = Self {
            id,
            provenance,
            data,
            designs,
            meta,
            description,
        };
        project.validate()?;
        Ok(project)
    }

    /// Returns this project's identifier.
    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    /// Returns its provenance graph.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Returns its data domain.
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Returns its design domain.
    pub fn designs(&self) -> &Designs {
        &self.designs
    }

    /// Returns its metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns its optional description.
    pub fn description(&self) -> Option<&Text> {
        self.description.as_ref()
    }

    fn validate(&self) -> Result<()> {
        self.data.validate_against(&self.provenance)?;
        self.designs.validate_against(&self.provenance)
    }
}
