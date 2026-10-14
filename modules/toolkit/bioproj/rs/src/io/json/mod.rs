//! One versioned JSON document per released project.
//!
//! Writers publish a complete document after serialization succeeds.

mod v0_0_1;

use super::fsutil;
use crate::io::Schema;
use crate::{Error, Project, Result};
use serde::Deserialize;
use serde_json::value::RawValue;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Reads and validates a project document without accessing its Asset files.
/// Relative Asset locations use the document's directory as their root.
pub fn read(path: impl AsRef<Path>) -> Result<Project> {
    let path = path.as_ref();
    let file =
        File::open(path).map_err(|error| Error::io("open project document", Some(path), error))?;
    let document: DocumentFromWire = serde_json::from_reader(BufReader::new(file))
        .map_err(|error| Error::json("parse project JSON", error))?;
    match document.schema {
        v0_0_1::VERSION => v0_0_1::decode(document, &fsutil::manifest_root(path)?),
        schema => Err(Error::UnsupportedSchema {
            found: schema.to_string(),
            supported: Schema::CURRENT.to_string(),
        }),
    }
}

/// Writes JSON, replacing the destination after the complete document is ready.
///
/// Asset files are not accessed. Relative paths are rendered for the output directory.
pub fn write(path: impl AsRef<Path>, project: &Project, pretty: bool) -> Result<()> {
    let destination = fsutil::manifest_root(path.as_ref())?;
    let bytes = v0_0_1::encode(project, pretty, &destination)?;
    fsutil::replace(path.as_ref(), &bytes)
}

// Keep domain payloads raw until their schema has been selected. RawValue also
// preserves duplicate fields for the domain deserializers to reject.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentFromWire {
    schema: Schema,
    project: Box<RawValue>,
    provenance: Box<RawValue>,
    data: Box<RawValue>,
    design: Box<RawValue>,
}
