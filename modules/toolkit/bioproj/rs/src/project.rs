use crate::data::Data;
use crate::primitives::define_entity_id;
use crate::provenance::Provenance;
use crate::validation;
use crate::{Designs, Meta, NonEmpty};
use eyre::Result;

define_entity_id!(ProjectId, "The identifier of a [`crate::Project`].");

/// A complete, immutable released biological-data project.
///
/// `Project` owns the released provenance, data, and design domains.
/// It is deliberately distinct from [`crate::Manifest`]: storage adapters,
/// schema versions, and authoring lifecycle state do not belong here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project {
    id: ProjectId,
    provenance: Provenance,
    data: Data,
    designs: Designs,
    meta: Meta,
    description: Option<NonEmpty<String>>,
}

impl Project {
    /// Constructs and validates a complete released project.
    pub fn new(
        id: ProjectId,
        provenance: Provenance,
        data: Data,
        designs: Designs,
        meta: Meta,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Self::from_parts(
            id,
            provenance,
            data,
            designs,
            meta,
            description
                .map(|description| NonEmpty::new(description.into()))
                .transpose()?,
        )
    }

    pub(crate) fn from_parts(
        id: ProjectId,
        provenance: Provenance,
        data: Data,
        designs: Designs,
        meta: Meta,
        description: Option<NonEmpty<String>>,
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

    fn validate(&self) -> Result<()> {
        self.data.validate_against(&self.provenance)?;
        self.designs.validate_against(&self.provenance)?;
        validation::unique_ids(
            "project",
            std::iter::once(self.id.as_untyped())
                .chain(self.provenance.ids())
                .chain(self.data.ids())
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

    /// Returns the project's resolved data domain.
    pub fn data(&self) -> &Data {
        &self.data
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
    pub fn description(&self) -> Option<&NonEmpty<String>> {
        self.description.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::{Project, ProjectId};
    use crate::Designs;
    use crate::data::asset::fastq::{Fastq, FastqId};
    use crate::data::dataset::fastq::{Single, SingleId, SingleInput};
    use crate::data::{Asset, Data, Dataset};
    use crate::provenance::acquisition::illumina::{SingleEndSequencing, SingleEndSequencingId};
    use crate::provenance::library::p5p7;
    use crate::provenance::{Acquisition, Library, Provenance, Sample, SampleId, Source, SourceId};

    fn domains(dataset_id: &str) -> (Provenance, Data, Designs) {
        let source_id = SourceId::new("SRC1").unwrap();
        let sample_id = SampleId::new("SMP1").unwrap();
        let library_id = p5p7::LibraryId::new("LIB1").unwrap();
        let acquisition_id = SingleEndSequencingId::new("ACQ1").unwrap();
        let provenance = Provenance::new(
            [Source::new(
                source_id.clone(),
                "Homo sapiens",
                Default::default(),
                None::<String>,
            )
            .unwrap()],
            [Sample::new(
                sample_id.clone(),
                [source_id],
                Default::default(),
                None::<String>,
            )
            .unwrap()],
            [Library::P5P7(
                p5p7::Library::new(
                    library_id.clone(),
                    [sample_id],
                    p5p7::Input::FromDna,
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            )],
            [Acquisition::IlluminaSingleEndSequencing(
                SingleEndSequencing::new(
                    acquisition_id.clone(),
                    [library_id],
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            )],
        )
        .unwrap();
        let assets = [Asset::Fastq(
            Fastq::new(
                FastqId::new("AST1").unwrap(),
                "file:reads.fq.gz",
                Default::default(),
                None::<String>,
            )
            .unwrap(),
        )];
        let datasets = [Dataset::Fastq(
            Single::new(
                SingleId::new(dataset_id).unwrap(),
                SingleInput::IlluminaSingleEndSequencing(acquisition_id),
                [FastqId::new("AST1").unwrap()],
                Default::default(),
                None::<String>,
            )
            .unwrap(),
        )];
        let data = Data::new(&provenance, assets, datasets).unwrap();
        let designs = Designs::new(&provenance, Vec::new(), Vec::new()).unwrap();
        (provenance, data, designs)
    }

    #[test]
    fn owns_a_complete_endpoint_description() {
        let (provenance, data, designs) = domains("DATA1");
        let project = Project::new(
            ProjectId::new("PROJ1").unwrap(),
            provenance,
            data,
            designs,
            Default::default(),
            None::<String>,
        )
        .unwrap();

        assert_eq!(project.data().datasets().len(), 1);
    }

    #[test]
    fn project_id_is_globally_unique() {
        let (provenance, data, designs) = domains("PROJ1");
        assert!(
            Project::new(
                ProjectId::new("PROJ1").unwrap(),
                provenance,
                data,
                designs,
                Default::default(),
                None::<String>,
            )
            .is_err()
        );
    }
}
