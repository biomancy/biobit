use crate::asset::UnresolvedAssets;
use crate::design::UnresolvedDesigns;
use crate::primitives::define_entity_id;
use crate::provenance::Provenance;
use crate::validation;
use crate::{Assets, Designs, Meta, MetaVal};
use eyre::Result;
use serde::{Deserialize, Deserializer, Serialize};

define_entity_id!(ProjectId, "The identifier of a [`crate::Project`].");

/// A complete, immutable released biological-data project.
///
/// `Project` owns the released provenance graph and its asset and design
/// domains. It is deliberately distinct from a future workspace manifest:
/// storage adapters, schema versions, and authoring lifecycle state do not
/// belong to this in-memory model.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Project {
    id: ProjectId,
    provenance: Provenance,
    assets: Assets,
    #[serde(rename = "design")]
    designs: Designs,
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    meta: Meta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl Project {
    /// Constructs and validates a complete released project.
    pub fn new(
        id: ProjectId,
        provenance: Provenance,
        assets: Assets,
        designs: Designs,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<MetaVal>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Self::from_parts(
            id,
            provenance,
            assets,
            designs,
            Meta::new(meta)?,
            description.map(Into::into),
        )
    }

    fn from_parts(
        id: ProjectId,
        provenance: Provenance,
        assets: Assets,
        designs: Designs,
        meta: Meta,
        description: Option<String>,
    ) -> Result<Self> {
        let project = Self {
            id,
            provenance,
            assets,
            designs,
            meta,
            description,
        };
        project.validate()?;
        Ok(project)
    }

    fn validate(&self) -> Result<()> {
        self.assets.validate_against(&self.provenance)?;
        self.designs.validate_against(&self.provenance)?;
        validation::unique_ids(
            "project",
            std::iter::once(self.id.as_id())
                .chain(self.provenance.ids())
                .chain(self.assets.ids())
                .chain(self.designs.ids()),
        )
    }

    /// Returns this project's identifier.
    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    /// Returns the physical and technical provenance graph.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Returns the project's physical data artifacts.
    pub fn assets(&self) -> &Assets {
        &self.assets
    }

    /// Returns the project's experimental-design overlay.
    pub fn designs(&self) -> &Designs {
        &self.designs
    }

    /// Returns auxiliary, non-structural metadata.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Returns the optional human-readable description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UnresolvedProject {
    id: ProjectId,
    provenance: Provenance,
    assets: UnresolvedAssets,
    #[serde(rename = "design")]
    designs: UnresolvedDesigns,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<String>,
}

impl<'de> Deserialize<'de> for Project {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let UnresolvedProject {
            id,
            provenance,
            assets,
            designs,
            meta,
            description,
        } = UnresolvedProject::deserialize(deserializer)?;
        let assets = assets
            .resolve(&provenance)
            .map_err(serde::de::Error::custom)?;
        let designs = designs
            .resolve(&provenance)
            .map_err(serde::de::Error::custom)?;
        Self::from_parts(id, provenance, assets, designs, meta, description)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{Project, ProjectId};
    use crate::asset::fastq::{Fastq, FastqInput};
    use crate::design::{Design, DesignId, DesignUnit, DesignUnitId, Designs, UnitSet};
    use crate::provenance::assay::illumina::{
        SequencingInput, SingleEndSequencing, SingleEndSequencingId,
    };
    use crate::provenance::library::illumina::{DnaLibrary, DnaLibraryId};
    use crate::provenance::run::illumina::{
        SingleEndSequencing as SingleEndRun, SingleEndSequencingId as SingleEndRunId,
    };
    use crate::provenance::{Assay, Library, Provenance, Run, Sample, SampleId, Source, SourceId};
    use crate::{Asset, AssetId, Assets, Id};

    fn provenance(run: &str) -> Provenance {
        let source_id = SourceId::new("SRC1").unwrap();
        let sample_id = SampleId::new("SMP1").unwrap();
        let library_id = DnaLibraryId::new("LIB1").unwrap();
        let assay_id = SingleEndSequencingId::new("ASY1").unwrap();
        let run_id = SingleEndRunId::new(run).unwrap();

        Provenance::new(
            [Source::new(
                source_id.clone(),
                "Homo sapiens",
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .unwrap()],
            [Sample::new(
                sample_id.clone(),
                [source_id],
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .unwrap()],
            [Library::IlluminaDna(
                DnaLibrary::new(
                    library_id.clone(),
                    [sample_id],
                    ["DNA"],
                    ["none"],
                    Vec::<(String, String)>::new(),
                    None::<String>,
                )
                .unwrap(),
            )],
            [Assay::IlluminaSingleEndSequencing(
                SingleEndSequencing::new(
                    assay_id.clone(),
                    SequencingInput::Dna(library_id),
                    Vec::<(String, String)>::new(),
                    None::<String>,
                )
                .unwrap(),
            )],
            [Run::IlluminaSingleEndSequencing(
                SingleEndRun::new(
                    run_id,
                    assay_id,
                    Vec::<(String, String)>::new(),
                    None::<String>,
                )
                .unwrap(),
            )],
        )
        .unwrap()
    }

    fn assets(provenance: &Provenance, run: &str) -> Assets {
        Assets::new(
            provenance,
            [Asset::Fastq(
                Fastq::new(
                    AssetId::new("AST1").unwrap(),
                    FastqInput::IlluminaSingleEndSequencing(SingleEndRunId::new(run).unwrap()),
                    ["s3://bucket/read.fq.gz"],
                    Vec::<(String, String)>::new(),
                    None::<String>,
                )
                .unwrap(),
            )],
        )
        .unwrap()
    }

    fn designs(provenance: &Provenance, unit: &str) -> Designs {
        Designs::new(
            provenance,
            [DesignUnit::new(
                DesignUnitId::new(unit).unwrap(),
                [Id::new("ASY1").unwrap()],
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .unwrap()],
            [Design::UnitSet(
                UnitSet::new(
                    DesignId::new("DES1").unwrap(),
                    [DesignUnitId::new(unit).unwrap()],
                    Vec::<(String, String)>::new(),
                    None::<String>,
                )
                .unwrap(),
            )],
        )
        .unwrap()
    }

    #[test]
    fn serializes_and_deserializes_a_complete_project() {
        let provenance = provenance("RUN1");
        let project = Project::new(
            ProjectId::new("PROJ1").unwrap(),
            provenance.clone(),
            assets(&provenance, "RUN1"),
            designs(&provenance, "UNIT1"),
            [("public", true)],
            Some("An example project"),
        )
        .unwrap();

        let serialized = serde_json::to_value(&project).unwrap();
        assert_eq!(serialized["id"], "PROJ1");
        assert_eq!(serialized["assets"]["assets"][0]["type"], "Fastq");
        assert_eq!(serialized["design"]["designs"][0]["type"], "UnitSet");
        assert_eq!(
            serde_json::from_value::<Project>(serialized).unwrap(),
            project
        );
    }

    #[test]
    fn rejects_ids_shared_between_assets_and_design() {
        let provenance = provenance("RUN1");
        assert!(
            Project::new(
                ProjectId::new("PROJ1").unwrap(),
                provenance.clone(),
                assets(&provenance, "RUN1"),
                designs(&provenance, "AST1"),
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .is_err()
        );
    }

    #[test]
    fn revalidates_domains_against_its_owned_provenance() {
        let owned_provenance = provenance("RUN_OWNED");
        let other_provenance = provenance("RUN_OTHER");
        assert!(
            Project::new(
                ProjectId::new("PROJ1").unwrap(),
                owned_provenance.clone(),
                assets(&other_provenance, "RUN_OTHER"),
                designs(&owned_provenance, "UNIT1"),
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .is_err()
        );
    }
}
