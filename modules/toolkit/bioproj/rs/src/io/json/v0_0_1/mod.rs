//! Encoding and decoding for the draft 0.0.1 project document.

mod data;
mod design;
mod provenance;

use super::{DocumentFromWire, Schema};
use crate::{Error, Meta, Project, ProjectId, Result, Root, Text};
use data::{DataFromWire, DataToWire};
use design::{DesignsFromWire, DesignsToWire};
use provenance::{ProvenanceFromWire, ProvenanceToWire};
use serde::{Deserialize, Serialize};

pub(super) const VERSION: Schema = Schema::new(0, 0, 1);

pub(super) fn decode(document: DocumentFromWire, root: &Root) -> Result<Project> {
    let project: ProjectFromWire = serde_json::from_str(document.project.get())
        .map_err(|error| Error::json("parse the project header", error))?;
    let provenance: ProvenanceFromWire = serde_json::from_str(document.provenance.get())
        .map_err(|error| Error::json("parse provenance", error))?;
    let data: DataFromWire = serde_json::from_str(document.data.get())
        .map_err(|error| Error::json("parse data", error))?;
    let design: DesignsFromWire = serde_json::from_str(document.design.get())
        .map_err(|error| Error::json("parse designs", error))?;
    let provenance = provenance.resolve()?;
    let data = data.resolve(root)?;
    let design = design.resolve(&provenance)?;
    Project::new(
        project.id,
        provenance,
        data,
        design,
        project.meta,
        project.description,
    )
}

pub(super) fn encode(project: &Project, pretty: bool, destination: &Root) -> Result<Vec<u8>> {
    let document = DocumentToWire {
        schema: VERSION,
        project: ProjectToWire {
            id: project.id(),
            meta: project.meta(),
            description: project.description(),
        },
        provenance: ProvenanceToWire::new(project.provenance()),
        data: DataToWire::new(project.data(), destination)?,
        design: DesignsToWire::new(project.designs()),
    };
    let mut bytes = if pretty {
        serde_json::to_vec_pretty(&document)
    } else {
        serde_json::to_vec(&document)
    }
    .map_err(|error| Error::json("serialize project JSON", error))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectFromWire {
    id: ProjectId,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<Text>,
}

#[derive(Serialize)]
struct DocumentToWire<'a> {
    schema: Schema,
    project: ProjectToWire<'a>,
    provenance: ProvenanceToWire<'a>,
    data: DataToWire<'a>,
    design: DesignsToWire<'a>,
}

#[derive(Serialize)]
struct ProjectToWire<'a> {
    id: &'a ProjectId,
    #[serde(skip_serializing_if = "Meta::is_empty")]
    meta: &'a Meta,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a Text>,
}
