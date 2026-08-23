use super::json;
use crate::data::Data;
use crate::provenance::Provenance;
use crate::{Designs, Uri};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A supported domain-storage adapter.
///
/// Adapters are intentionally a closed enum. New storage mechanisms are added
/// as explicit variants rather than through a dynamically registered trait.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Adapter {
    /// A JSON domain payload.
    Json,
}

/// A domain payload location and the adapter that reads it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    adapter: Adapter,
    uri: Uri,
}

impl Location {
    /// Creates a domain payload location.
    pub fn new(adapter: Adapter, uri: Uri) -> Self {
        Self { adapter, uri }
    }

    /// Returns the closed adapter selected for this payload.
    pub fn adapter(&self) -> Adapter {
        self.adapter
    }

    /// Returns the payload's workspace-local file locator.
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub(crate) fn load_provenance(&self, directory: &Path) -> Result<Provenance> {
        let path = self.uri.resolve_against(directory);
        match self.adapter {
            Adapter::Json => json::load_provenance(&path),
        }
    }

    pub(crate) fn load_data(&self, directory: &Path, provenance: &Provenance) -> Result<Data> {
        let path = self.uri.resolve_against(directory);
        match self.adapter {
            Adapter::Json => json::load_data(&path, provenance),
        }
    }

    pub(crate) fn load_designs(
        &self,
        directory: &Path,
        provenance: &Provenance,
    ) -> Result<Designs> {
        let path = self.uri.resolve_against(directory);
        match self.adapter {
            Adapter::Json => json::load_designs(&path, provenance),
        }
    }
}
