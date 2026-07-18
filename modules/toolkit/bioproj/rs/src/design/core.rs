use super::DesignId;
use crate::{Meta, MetaVal};
use eyre::Result;

/// Fields shared by concrete experimental design topologies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DesignCore {
    pub(super) id: DesignId,
    pub(super) meta: Meta,
    pub(super) description: Option<String>,
}

impl DesignCore {
    pub(super) fn new(
        id: DesignId,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            meta: Meta::new(meta)?,
            description: description.map(Into::into),
        })
    }
}
