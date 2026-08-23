use super::DesignId;
use crate::{Meta, NonEmpty};
use eyre::Result;

/// Fields shared by concrete experimental design topologies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DesignCore {
    pub(super) id: DesignId,
    pub(super) meta: Meta,
    pub(super) description: Option<NonEmpty<String>>,
}

impl DesignCore {
    pub(super) fn new(
        id: DesignId,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            meta,
            description: description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        })
    }
}
