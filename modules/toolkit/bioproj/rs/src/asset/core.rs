use super::AssetId;
use crate::{Meta, MetaVal};
use eyre::Result;

/// Fields common to concrete asset formats.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct AssetCore<R> {
    pub(super) id: AssetId,
    pub(super) run: R,
    pub(super) meta: Meta,
    pub(super) description: Option<String>,
}

impl<R> AssetCore<R> {
    pub(super) fn new(
        id: AssetId,
        run: R,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            run,
            meta: Meta::new(meta)?,
            description: description.map(Into::into),
        })
    }
}
