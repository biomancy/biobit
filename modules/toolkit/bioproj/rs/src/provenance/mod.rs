//! The physical and technical provenance DAG.

pub mod assay;
pub mod library;
pub mod run;
pub mod sample;
pub mod source;

pub use assay::Assay;
pub use library::Library;
pub use run::Run;
pub use sample::{Sample, SampleId};
pub use source::{Source, SourceId};

use self::assay::UnresolvedAssay;
use self::run::UnresolvedRun;
use crate::Id;
use crate::validation;
use eyre::{Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

/// The resolved, immutable provenance graph for a released description.
///
/// This initial graph models biological material and its immediate acquisition
/// contract: [`Source`] <- [`Sample`] <- [`Library`] <- [`Assay`] <- [`Run`].
/// A library retains its concrete material type, while each child owns the
/// explicit set of concrete parent ID types it can consume.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenance {
    sources: BTreeMap<SourceId, Source>,
    samples: BTreeMap<SampleId, Sample>,
    libraries: BTreeMap<Id, Library>,
    assays: BTreeMap<Id, Assay>,
    runs: BTreeMap<Id, Run>,
}

impl Provenance {
    /// Constructs and validates a released provenance graph.
    pub fn new(
        sources: impl IntoIterator<Item = Source>,
        samples: impl IntoIterator<Item = Sample>,
        libraries: impl IntoIterator<Item = Library>,
        assays: impl IntoIterator<Item = Assay>,
        runs: impl IntoIterator<Item = Run>,
    ) -> Result<Self> {
        let sources: Vec<_> = sources.into_iter().collect();
        let samples: Vec<_> = samples.into_iter().collect();
        let libraries: Vec<_> = libraries.into_iter().collect();
        let assays: Vec<_> = assays.into_iter().collect();
        let runs: Vec<_> = runs.into_iter().collect();

        validation::unique_ids(
            "provenance",
            sources
                .iter()
                .map(|source| source.id().as_id())
                .chain(samples.iter().map(|sample| sample.id().as_id()))
                .chain(libraries.iter().map(Library::id))
                .chain(assays.iter().map(Assay::id))
                .chain(runs.iter().map(Run::id)),
        )?;

        let sources = sources
            .into_iter()
            .map(|source| (source.id().clone(), source))
            .collect();
        let samples = samples
            .into_iter()
            .map(|sample| (sample.id().clone(), sample))
            .collect();
        let libraries = libraries
            .into_iter()
            .map(|library| (library.id().clone(), library))
            .collect();
        let assays = assays
            .into_iter()
            .map(|assay| (assay.id().clone(), assay))
            .collect();
        let runs = runs
            .into_iter()
            .map(|run| (run.id().clone(), run))
            .collect();

        validate_material_references(&sources, &samples, &libraries)?;
        validate_assay_references(&libraries, &assays)?;
        validate_run_references(&assays, &runs)?;

        Ok(Self {
            sources,
            samples,
            libraries,
            assays,
            runs,
        })
    }

    /// Returns sources keyed by their typed IDs.
    pub fn sources(&self) -> &BTreeMap<SourceId, Source> {
        &self.sources
    }

    /// Returns samples keyed by their typed IDs.
    pub fn samples(&self) -> &BTreeMap<SampleId, Sample> {
        &self.samples
    }

    /// Returns concrete libraries keyed by their globally unique raw IDs.
    pub fn libraries(&self) -> &BTreeMap<Id, Library> {
        &self.libraries
    }

    /// Returns concrete assays keyed by their globally unique raw IDs.
    pub fn assays(&self) -> &BTreeMap<Id, Assay> {
        &self.assays
    }

    /// Returns concrete runs keyed by their globally unique raw IDs.
    pub fn runs(&self) -> &BTreeMap<Id, Run> {
        &self.runs
    }

    /// Finds a source by its typed ID.
    pub fn source(&self, id: &SourceId) -> Option<&Source> {
        self.sources.get(id)
    }

    /// Finds a sample by its typed ID.
    pub fn sample(&self, id: &SampleId) -> Option<&Sample> {
        self.samples.get(id)
    }

    /// Finds a concrete library by its globally unique raw ID.
    pub fn library(&self, id: &Id) -> Option<&Library> {
        self.libraries.get(id)
    }

    /// Finds an assay by its globally unique raw ID.
    pub fn assay(&self, id: &Id) -> Option<&Assay> {
        self.assays.get(id)
    }

    /// Finds a concrete run by its globally unique raw ID.
    pub fn run(&self, id: &Id) -> Option<&Run> {
        self.runs.get(id)
    }

    /// Iterates over all IDs already occupied by this provenance graph.
    pub(crate) fn ids(&self) -> impl Iterator<Item = &Id> {
        self.sources
            .values()
            .map(|source| source.id().as_id())
            .chain(self.samples.values().map(|sample| sample.id().as_id()))
            .chain(self.libraries.keys())
            .chain(self.assays.keys())
            .chain(self.runs.keys())
    }
}

fn validate_material_references(
    sources: &BTreeMap<SourceId, Source>,
    samples: &BTreeMap<SampleId, Sample>,
    libraries: &BTreeMap<Id, Library>,
) -> Result<()> {
    for sample in samples.values() {
        for source_id in sample.sources() {
            if !sources.contains_key(source_id) {
                bail!(
                    "Sample '{}' references unknown Source '{source_id}'",
                    sample.id()
                );
            }
        }
    }

    for library in libraries.values() {
        for sample_id in library.samples() {
            if !samples.contains_key(sample_id) {
                bail!(
                    "Library '{}' references unknown Sample '{sample_id}'",
                    library.id()
                );
            }
        }
    }
    Ok(())
}

fn validate_assay_references(
    libraries: &BTreeMap<Id, Library>,
    assays: &BTreeMap<Id, Assay>,
) -> Result<()> {
    for assay in assays.values() {
        assay.validate_references(libraries)?;
    }
    Ok(())
}

fn validate_run_references(assays: &BTreeMap<Id, Assay>, runs: &BTreeMap<Id, Run>) -> Result<()> {
    for run in runs.values() {
        run.validate_references(assays)?;
    }
    Ok(())
}

#[derive(Serialize)]
struct SerializedProvenance<'a> {
    sources: Vec<&'a Source>,
    samples: Vec<&'a Sample>,
    libraries: Vec<&'a Library>,
    assays: Vec<&'a Assay>,
    runs: Vec<&'a Run>,
}

impl Serialize for Provenance {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedProvenance {
            sources: self.sources.values().collect(),
            samples: self.samples.values().collect(),
            libraries: self.libraries.values().collect(),
            assays: self.assays.values().collect(),
            runs: self.runs.values().collect(),
        }
        .serialize(serializer)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeserializedProvenance {
    sources: Vec<Source>,
    samples: Vec<Sample>,
    libraries: Vec<Library>,
    assays: Vec<UnresolvedAssay>,
    runs: Vec<UnresolvedRun>,
}

impl<'de> Deserialize<'de> for Provenance {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let provenance = DeserializedProvenance::deserialize(deserializer)?;
        validation::unique_ids(
            "deserialized libraries",
            provenance.libraries.iter().map(Library::id),
        )
        .map_err(serde::de::Error::custom)?;

        let library_index = provenance
            .libraries
            .iter()
            .map(|library| (library.id().clone(), library.clone()))
            .collect();
        let assays = provenance
            .assays
            .into_iter()
            .map(|assay| assay.resolve(&library_index))
            .collect::<Result<Vec<_>>>()
            .map_err(serde::de::Error::custom)?;

        validation::unique_ids("deserialized assays", assays.iter().map(Assay::id))
            .map_err(serde::de::Error::custom)?;

        let assay_index = assays
            .iter()
            .map(|assay| (assay.id().clone(), assay.clone()))
            .collect();
        let runs = provenance
            .runs
            .into_iter()
            .map(|run| run.resolve(&assay_index))
            .collect::<Result<Vec<_>>>()
            .map_err(serde::de::Error::custom)?;

        Self::new(
            provenance.sources,
            provenance.samples,
            provenance.libraries,
            assays,
            runs,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::assay::illumina::{
        PairedEndSequencing, PairedEndSequencingId, SequencingInput, SingleEndSequencing,
        SingleEndSequencingId,
    };
    use super::library::illumina::{CdnaLibrary, CdnaLibraryId, DnaLibrary, DnaLibraryId};
    use super::library::strandedness::Strandedness;
    use super::run::illumina::{
        PairedEndSequencing as PairedEndRun, PairedEndSequencingId as PairedEndRunId,
        SingleEndSequencing as SingleEndRun, SingleEndSequencingId as SingleEndRunId,
    };
    use super::{Assay, Library, Provenance, Run, Sample, SampleId, Source, SourceId};

    fn source_id(id: &str) -> SourceId {
        SourceId::new(id).unwrap()
    }

    fn sample_id(id: &str) -> SampleId {
        SampleId::new(id).unwrap()
    }

    fn dna_library_id(id: &str) -> DnaLibraryId {
        DnaLibraryId::new(id).unwrap()
    }

    fn cdna_library_id(id: &str) -> CdnaLibraryId {
        CdnaLibraryId::new(id).unwrap()
    }

    fn source(id: &str) -> Source {
        Source::new(
            source_id(id),
            "Homo sapiens",
            [("kind", "donor")],
            None::<String>,
        )
        .unwrap()
    }

    fn sample(id: &str, sources: impl IntoIterator<Item = SourceId>) -> Sample {
        Sample::new(sample_id(id), sources, [("kind", "tissue")], None::<String>).unwrap()
    }

    fn dna_library(id: &str, samples: impl IntoIterator<Item = SampleId>) -> Library {
        Library::IlluminaDna(
            DnaLibrary::new(
                dna_library_id(id),
                samples,
                ["DNA"],
                ["none"],
                [("kind", "dna")],
                None::<String>,
            )
            .unwrap(),
        )
    }

    fn cdna_library(id: &str, samples: impl IntoIterator<Item = SampleId>) -> Library {
        Library::IlluminaCdna(
            CdnaLibrary::new(
                cdna_library_id(id),
                samples,
                ["cDNA"],
                ["poly-A"],
                Strandedness::Forward,
                [("kind", "rna")],
                None::<String>,
            )
            .unwrap(),
        )
    }

    #[test]
    fn dna_and_cdna_libraries_can_feed_both_illumina_layouts() {
        let source = source_id("SRC1");
        let sample = sample_id("SMP1");
        let dna = dna_library_id("LIB_DNA");
        let cdna = cdna_library_id("LIB_CDNA");
        let single_end_dna = SingleEndSequencingId::new("ASY_SE_DNA").unwrap();
        let paired_end_dna = PairedEndSequencingId::new("ASY_PE_DNA").unwrap();
        let single_end_cdna = SingleEndSequencingId::new("ASY_SE_CDNA").unwrap();
        let paired_end_cdna = PairedEndSequencingId::new("ASY_PE_CDNA").unwrap();
        let single_end_run = SingleEndRunId::new("RUN_SE_DNA").unwrap();
        let paired_end_run = PairedEndRunId::new("RUN_PE_DNA").unwrap();

        let provenance = Provenance::new(
            [self::source("SRC1")],
            [self::sample("SMP1", [source])],
            [
                dna_library("LIB_DNA", [sample.clone()]),
                cdna_library("LIB_CDNA", [sample]),
            ],
            [
                Assay::IlluminaSingleEndSequencing(
                    SingleEndSequencing::new(
                        single_end_dna.clone(),
                        SequencingInput::Dna(dna.clone()),
                        Vec::<(String, String)>::new(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
                Assay::IlluminaPairedEndSequencing(
                    PairedEndSequencing::new(
                        paired_end_dna.clone(),
                        SequencingInput::Dna(dna.clone()),
                        Vec::<(String, String)>::new(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
                Assay::IlluminaSingleEndSequencing(
                    SingleEndSequencing::new(
                        single_end_cdna.clone(),
                        SequencingInput::Cdna(cdna.clone()),
                        Vec::<(String, String)>::new(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
                Assay::IlluminaPairedEndSequencing(
                    PairedEndSequencing::new(
                        paired_end_cdna.clone(),
                        SequencingInput::Cdna(cdna.clone()),
                        Vec::<(String, String)>::new(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ],
            [
                Run::IlluminaSingleEndSequencing(
                    SingleEndRun::new(
                        single_end_run.clone(),
                        single_end_dna.clone(),
                        Vec::<(String, String)>::new(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
                Run::IlluminaPairedEndSequencing(
                    PairedEndRun::new(
                        paired_end_run.clone(),
                        paired_end_dna.clone(),
                        Vec::<(String, String)>::new(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        assert!(matches!(
            provenance.library(dna.as_id()),
            Some(Library::IlluminaDna(_))
        ));
        assert!(matches!(
            provenance.library(cdna.as_id()),
            Some(Library::IlluminaCdna(_))
        ));
        assert!(matches!(
            provenance.assay(single_end_dna.as_id()),
            Some(Assay::IlluminaSingleEndSequencing(_))
        ));
        assert!(matches!(
            provenance.assay(paired_end_dna.as_id()),
            Some(Assay::IlluminaPairedEndSequencing(_))
        ));
        assert!(matches!(
            provenance.assay(single_end_cdna.as_id()),
            Some(Assay::IlluminaSingleEndSequencing(_))
        ));
        assert!(matches!(
            provenance.assay(paired_end_cdna.as_id()),
            Some(Assay::IlluminaPairedEndSequencing(_))
        ));
        assert!(matches!(
            provenance.run(single_end_run.as_id()),
            Some(Run::IlluminaSingleEndSequencing(_))
        ));
        assert!(matches!(
            provenance.run(paired_end_run.as_id()),
            Some(Run::IlluminaPairedEndSequencing(_))
        ));
    }

    #[test]
    fn rejects_typed_input_that_resolves_to_the_wrong_library_variant() {
        let source = source_id("SRC1");
        let sample = sample_id("SMP1");

        assert!(
            Provenance::new(
                [self::source("SRC1")],
                [self::sample("SMP1", [source])],
                [cdna_library("LIB1", [sample])],
                [Assay::IlluminaSingleEndSequencing(
                    SingleEndSequencing::new(
                        SingleEndSequencingId::new("ASY1").unwrap(),
                        SequencingInput::Dna(dna_library_id("LIB1")),
                        Vec::<(String, String)>::new(),
                        None::<String>,
                    )
                    .unwrap(),
                )],
                Vec::<Run>::new(),
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_typed_run_parent_that_resolves_to_the_wrong_assay_variant() {
        let source = source_id("SRC1");
        let sample = sample_id("SMP1");
        let library = dna_library_id("LIB1");

        assert!(
            Provenance::new(
                [self::source("SRC1")],
                [self::sample("SMP1", [source])],
                [dna_library("LIB1", [sample])],
                [Assay::IlluminaSingleEndSequencing(
                    SingleEndSequencing::new(
                        SingleEndSequencingId::new("ASY1").unwrap(),
                        SequencingInput::Dna(library),
                        Vec::<(String, String)>::new(),
                        None::<String>,
                    )
                    .unwrap(),
                )],
                [Run::IlluminaPairedEndSequencing(
                    PairedEndRun::new(
                        PairedEndRunId::new("RUN1").unwrap(),
                        PairedEndSequencingId::new("ASY1").unwrap(),
                        Vec::<(String, String)>::new(),
                        None::<String>,
                    )
                    .unwrap(),
                )],
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_dangling_material_references() {
        assert!(
            Provenance::new(
                Vec::<Source>::new(),
                [self::sample("SMP1", [source_id("MISSING_SOURCE")])],
                Vec::<Library>::new(),
                Vec::<Assay>::new(),
                Vec::<Run>::new(),
            )
            .is_err()
        );

        assert!(
            Provenance::new(
                [self::source("SRC1")],
                Vec::<Sample>::new(),
                [dna_library("LIB1", [sample_id("MISSING_SAMPLE")])],
                Vec::<Assay>::new(),
                Vec::<Run>::new(),
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_ids_shared_between_entity_types() {
        assert!(
            Provenance::new(
                [self::source("DUP")],
                [self::sample("SMP1", [source_id("DUP")])],
                [dna_library("DUP", [sample_id("SMP1")])],
                Vec::<Assay>::new(),
                Vec::<Run>::new(),
            )
            .is_err()
        );
    }

    #[test]
    fn serializes_and_resolves_assay_input_types() {
        let provenance: Provenance = serde_json::from_str(
            r#"{
                "sources": [
                    {"id": "SRC1", "organism": "Homo sapiens"}
                ],
                "samples": [
                    {"id": "SMP1", "sources": ["SRC1"]}
                ],
                "libraries": [
                    {
                        "type": "IlluminaCdna",
                        "id": "LIB1",
                        "samples": ["SMP1"],
                        "molecule": ["cDNA"],
                        "selection": ["poly-A"],
                        "strandedness": "Unknown"
                    }
                ],
                "assays": [
                    {
                        "type": "IlluminaPairedEndSequencing",
                        "id": "ASY1",
                        "library": "LIB1"
                    }
                ],
                "runs": [
                    {
                        "type": "IlluminaPairedEndSequencing",
                        "id": "RUN1",
                        "assay": "ASY1"
                    }
                ]
            }"#,
        )
        .unwrap();

        let assay_id = PairedEndSequencingId::new("ASY1").unwrap();
        let assay = provenance.assay(assay_id.as_id()).unwrap();
        let Assay::IlluminaPairedEndSequencing(assay) = assay else {
            panic!("serialized paired-end Illumina assay resolved to a different variant");
        };
        assert_eq!(
            assay.library(),
            &SequencingInput::Cdna(cdna_library_id("LIB1"))
        );

        let run_id = PairedEndRunId::new("RUN1").unwrap();
        assert!(matches!(
            provenance.run(run_id.as_id()),
            Some(Run::IlluminaPairedEndSequencing(_))
        ));

        assert_eq!(
            serde_json::from_str::<Provenance>(&serde_json::to_string(&provenance).unwrap())
                .unwrap(),
            provenance
        );
    }
}
