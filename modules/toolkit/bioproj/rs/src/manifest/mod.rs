//! TOML workspace manifests and their closed storage-adapter orchestration.

mod adapter;
mod json;
mod schema;

pub use adapter::{Adapter, Location};
pub use schema::Schema;

use crate::{Meta, NonEmpty, Project, ProjectId};
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// The project-level fields stored in a manifest.
///
/// Unlike [`Project`], this header has no resolved graph. Its schema and
/// locations govern how the manifest loader obtains that graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectHeader {
    id: ProjectId,
    schema: Schema,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<NonEmpty<String>>,
}

impl ProjectHeader {
    /// Returns the released project's identifier.
    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    /// Returns the manifest schema selected for this project.
    pub fn schema(&self) -> Schema {
        self.schema
    }

    /// Returns auxiliary, non-structural project metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns the optional human-readable project description.
    pub fn description(&self) -> Option<&NonEmpty<String>> {
        self.description.as_ref()
    }
}

/// A primary `bioproj.toml` manifest.
///
/// The manifest is an orchestration record, not a second graph model. Loading
/// first selects a supported schema, then resolves provenance, data, and design
/// through their declared closed adapters before constructing a [`Project`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    project: ProjectHeader,
    provenance: Domain,
    data: Domain,
    design: Domain,
}

impl Manifest {
    /// Loads a manifest and resolves it into an immutable released project.
    pub fn load(path: impl AsRef<Path>) -> Result<Project> {
        let path = path.as_ref();
        let manifest = read(path)?;
        manifest.resolve(path)
    }

    /// Returns the manifest's project-level header.
    pub fn project(&self) -> &ProjectHeader {
        &self.project
    }

    /// Returns the provenance payload location.
    pub fn provenance_location(&self) -> &Location {
        &self.provenance.location
    }

    /// Returns the data payload location.
    pub fn data_location(&self) -> &Location {
        &self.data.location
    }

    /// Returns the design payload location.
    pub fn design_location(&self) -> &Location {
        &self.design.location
    }

    fn resolve(self, manifest_path: &Path) -> Result<Project> {
        let Self {
            project,
            provenance,
            data,
            design,
        } = self;
        project.schema.ensure_supported()?;

        let directory = manifest_directory(manifest_path);
        let provenance = provenance.location.load_provenance(directory)?;
        let data = data.location.load_data(directory, &provenance)?;
        let designs = design.location.load_designs(directory, &provenance)?;

        Project::from_parts(
            project.id,
            provenance,
            data,
            designs,
            project.meta,
            project.description,
        )
        .wrap_err("failed to assemble released project from manifest payloads")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Domain {
    location: Location,
}

fn read(path: &Path) -> Result<Manifest> {
    let contents = fs::read_to_string(path)
        .wrap_err_with(|| format!("failed to read manifest '{}'", path.display()))?;
    toml::from_str(&contents)
        .wrap_err_with(|| format!("failed to parse manifest '{}'", path.display()))
}

fn manifest_directory(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

#[cfg(test)]
mod tests {
    use super::{Manifest, Schema};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_WORKSPACE: AtomicUsize = AtomicUsize::new(0);

    struct TestWorkspace(PathBuf);

    impl TestWorkspace {
        fn new() -> Self {
            let nonce = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "biobit-bioproj-manifest-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn write(&self, name: &str, contents: &str) {
            fs::write(self.path().join(name), contents).unwrap();
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const PROVENANCE: &str = r#"{
        "sources": [{"id": "SRC1", "organism": "Homo sapiens"}],
        "samples": [{"id": "SMP1", "sources": ["SRC1"]}],
        "libraries": [{
            "type": "P5P7",
            "id": "LIB1",
            "samples": ["SMP1"],
            "input": {"type": "FromDna"}
        }],
        "acquisitions": [{
            "type": "IlluminaSingleEndSequencing",
            "id": "ACQ1",
            "libraries": ["LIB1"]
        }]
    }"#;

    const DATA: &str = r#"{
        "assets": [{
            "type": "Fastq",
            "id": "AST1",
            "location": "file:read.fq.gz"
        }],
        "datasets": [{
            "type": "Fastq",
            "id": "DATA1",
            "acquisition": "ACQ1",
            "assets": ["AST1"]
        }]
    }"#;

    const DESIGN: &str = r#"{
        "units": [{"id": "UNIT1", "acquisitions": ["ACQ1"]}],
        "designs": [{"type": "UnitSet", "id": "DES1", "units": ["UNIT1"]}]
    }"#;

    fn manifest(schema: &str) -> String {
        format!(
            r#"
                [project]
                id = "PROJ1"
                schema = "{schema}"
                description = "Example"
                meta = {{ public = true }}

                [provenance]
                location = {{ adapter = "json", uri = "file:provenance.json" }}

                [data]
                location = {{ adapter = "json", uri = "file:data.json" }}

                [design]
                location = {{ adapter = "json", uri = "file:design.json" }}
            "#
        )
    }

    fn populated_workspace() -> TestWorkspace {
        let workspace = TestWorkspace::new();
        workspace.write("provenance.json", PROVENANCE);
        workspace.write("data.json", DATA);
        workspace.write("design.json", DESIGN);
        workspace
    }

    #[test]
    fn loads_all_explicit_domain_payloads() {
        let workspace = populated_workspace();
        workspace.write("bioproj.toml", &manifest("0.0.1"));

        let project = Manifest::load(workspace.path().join("bioproj.toml")).unwrap();
        assert_eq!(project.id().as_untyped().as_str(), "PROJ1");
        assert_eq!(project.provenance().acquisitions().len(), 1);
        assert_eq!(project.data().assets().len(), 1);
        assert_eq!(project.data().datasets().len(), 1);
        assert_eq!(project.designs().designs().len(), 1);
    }

    #[test]
    fn accepts_explicit_empty_domain_payloads() {
        let workspace = populated_workspace();
        workspace.write(
            "provenance.json",
            r#"{"sources":[],"samples":[],"libraries":[],"acquisitions":[]}"#,
        );
        workspace.write("data.json", r#"{"assets": [], "datasets": []}"#);
        workspace.write("design.json", r#"{"units": [], "designs": []}"#);
        workspace.write("bioproj.toml", &manifest("0.0.1"));

        let project = Manifest::load(workspace.path().join("bioproj.toml")).unwrap();
        assert_eq!(project.data().assets().len(), 0);
        assert_eq!(project.data().datasets().len(), 0);
        assert_eq!(project.designs().units().len(), 0);
        assert_eq!(project.designs().designs().len(), 0);
    }

    #[test]
    fn requires_each_domain_section() {
        let workspace = populated_workspace();
        workspace.write(
            "bioproj.toml",
            r#"
                [project]
                id = "PROJ1"
                schema = "0.0.1"

                [provenance]
                location = { adapter = "json", uri = "file:provenance.json" }

                [data]
                location = { adapter = "json", uri = "file:data.json" }
            "#,
        );

        assert!(Manifest::load(workspace.path().join("bioproj.toml")).is_err());
    }

    #[test]
    fn dispatches_schema_before_reading_payloads() {
        let workspace = TestWorkspace::new();
        workspace.write("bioproj.toml", &manifest("1.0.0"));

        let error = Manifest::load(workspace.path().join("bioproj.toml")).unwrap_err();
        assert!(error.to_string().contains("unsupported bioproj schema"));
    }

    #[test]
    fn rejects_unknown_adapters() {
        let workspace = TestWorkspace::new();
        workspace.write(
            "bioproj.toml",
            &manifest("0.0.1").replace("adapter = \"json\"", "adapter = \"yaml\""),
        );

        assert!(Manifest::load(workspace.path().join("bioproj.toml")).is_err());
    }

    #[test]
    fn reports_a_missing_domain_payload() {
        let workspace = TestWorkspace::new();
        workspace.write("bioproj.toml", &manifest("0.0.1"));

        let error = Manifest::load(workspace.path().join("bioproj.toml")).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("failed to read provenance payload")
        );
    }

    #[test]
    fn reports_malformed_domain_json() {
        let workspace = populated_workspace();
        workspace.write("data.json", "{");
        workspace.write("bioproj.toml", &manifest("0.0.1"));

        let error = Manifest::load(workspace.path().join("bioproj.toml")).unwrap_err();
        assert!(error.to_string().contains("failed to parse data JSON"));
    }

    #[test]
    fn serializes_the_schema_as_a_string_in_toml() {
        #[derive(serde::Serialize)]
        struct SchemaHolder {
            schema: Schema,
        }

        let serialized = toml::to_string(&SchemaHolder {
            schema: Schema::CURRENT,
        })
        .unwrap();
        assert_eq!(serialized, "schema = \"0.0.1\"\n");
    }
}
