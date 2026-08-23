//! The physical and technical provenance DAG.

pub mod acquisition;
pub mod library;
mod lookup;
pub mod sample;
pub mod source;
mod validate;

pub use acquisition::{Acquisition, AcquisitionId, AcquisitionIdRef, AcquisitionKind};
pub use library::{Library, LibraryId, LibraryIdRef, LibraryKind};
pub use sample::{Sample, SampleId};
pub use source::{Source, SourceId};

use crate::primitives::UniqueMap;
use crate::{Lookup, UntypedId};
use eyre::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The resolved, immutable provenance graph for a released description.
///
/// This initial graph models biological material and its immediate acquisition
/// contract: [`Source`] <- [`Sample`] <- [`Library`] <- [`Acquisition`].
/// A library retains its concrete downstream interface and input path, while
/// each child owns the concrete parent ID types it can consume.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenance {
    sources: UniqueMap<SourceId, Source>,
    samples: UniqueMap<SampleId, Sample>,
    libraries: UniqueMap<UntypedId, Library>,
    acquisitions: UniqueMap<UntypedId, Acquisition>,
}

impl Provenance {
    /// Constructs and validates a released provenance graph.
    pub fn new(
        sources: impl IntoIterator<Item = Source>,
        samples: impl IntoIterator<Item = Sample>,
        libraries: impl IntoIterator<Item = Library>,
        acquisitions: impl IntoIterator<Item = Acquisition>,
    ) -> Result<Self> {
        let provenance = Self {
            sources: UniqueMap::try_from_iter(
                sources
                    .into_iter()
                    .map(|source| (source.id().clone(), source)),
            )?,
            samples: UniqueMap::try_from_iter(
                samples
                    .into_iter()
                    .map(|sample| (sample.id().clone(), sample)),
            )?,
            libraries: UniqueMap::try_from_iter(
                libraries
                    .into_iter()
                    .map(|library| (library.id().as_untyped().clone(), library)),
            )?,
            acquisitions: UniqueMap::try_from_iter(
                acquisitions
                    .into_iter()
                    .map(|acquisition| (acquisition.id().as_untyped().clone(), acquisition)),
            )?,
        };
        provenance.validate()?;
        Ok(provenance)
    }

    /// Iterates over all sources.
    pub fn sources(&self) -> impl ExactSizeIterator<Item = &Source> + '_ {
        self.sources.values()
    }

    /// Iterates over all samples.
    pub fn samples(&self) -> impl ExactSizeIterator<Item = &Sample> + '_ {
        self.samples.values()
    }

    /// Iterates over all concrete libraries.
    pub fn libraries(&self) -> impl ExactSizeIterator<Item = &Library> + '_ {
        self.libraries.values()
    }

    /// Iterates over all concrete acquisitions.
    pub fn acquisitions(&self) -> impl ExactSizeIterator<Item = &Acquisition> + '_ {
        self.acquisitions.values()
    }

    /// Performs a lookup whose result is determined by its typed ID.
    pub fn get<'a, K: ?Sized>(&'a self, key: &K) -> Option<<Self as Lookup<K>>::Found<'a>>
    where
        Self: Lookup<K>,
    {
        <Self as Lookup<K>>::lookup(self, key)
    }

    /// Finds a source by its untyped ID.
    pub fn source(&self, id: &UntypedId) -> Option<&Source> {
        self.sources.get(id)
    }

    /// Finds a sample by its untyped ID.
    pub fn sample(&self, id: &UntypedId) -> Option<&Sample> {
        self.samples.get(id)
    }

    /// Finds a library by its untyped ID.
    pub fn library(&self, id: &UntypedId) -> Option<&Library> {
        self.libraries.get(id)
    }

    /// Finds an acquisition by its untyped ID.
    pub fn acquisition(&self, id: &UntypedId) -> Option<&Acquisition> {
        self.acquisitions.get(id)
    }

    /// Iterates over all IDs already occupied by this provenance graph.
    pub(crate) fn ids(&self) -> impl Iterator<Item = &UntypedId> {
        self.sources
            .keys()
            .map(SourceId::as_untyped)
            .chain(self.samples.keys().map(SampleId::as_untyped))
            .chain(self.libraries.keys())
            .chain(self.acquisitions.keys())
    }

    fn validate(&self) -> Result<()> {
        validate::validate(self)
    }
}

#[derive(Serialize)]
struct SerializedProvenance<'a> {
    sources: Vec<&'a Source>,
    samples: Vec<&'a Sample>,
    libraries: Vec<&'a Library>,
    acquisitions: Vec<&'a Acquisition>,
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
            acquisitions: self.acquisitions.values().collect(),
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
    acquisitions: Vec<Acquisition>,
}

impl<'de> Deserialize<'de> for Provenance {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let provenance = DeserializedProvenance::deserialize(deserializer)?;
        Self::new(
            provenance.sources,
            provenance.samples,
            provenance.libraries,
            provenance.acquisitions,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::acquisition::illumina::{
        PairedEndSequencing, PairedEndSequencingId, SingleEndSequencing, SingleEndSequencingId,
    };
    use super::library::p5p7::{Input, Library as P5P7, LibraryId};
    use super::library::strandedness::Strandedness;
    use super::{
        Acquisition, AcquisitionId, AcquisitionIdRef, Library, LibraryId as AnyLibraryId,
        LibraryIdRef, Provenance, Sample, SampleId, Source, SourceId,
    };

    fn source_id(id: &str) -> SourceId {
        SourceId::new(id).unwrap()
    }

    fn sample_id(id: &str) -> SampleId {
        SampleId::new(id).unwrap()
    }

    fn library_id(id: &str) -> LibraryId {
        LibraryId::new(id).unwrap()
    }

    fn source(id: &str) -> Source {
        Source::new(
            source_id(id),
            "Homo sapiens",
            Default::default(),
            None::<String>,
        )
        .unwrap()
    }

    fn sample(id: &str, sources: impl IntoIterator<Item = SourceId>) -> Sample {
        Sample::new(sample_id(id), sources, Default::default(), None::<String>).unwrap()
    }

    fn library(id: &str, samples: impl IntoIterator<Item = SampleId>, input: Input) -> Library {
        P5P7::new(
            library_id(id),
            samples,
            input,
            Default::default(),
            None::<String>,
        )
        .unwrap()
        .into()
    }

    #[test]
    fn dna_and_rna_inputs_can_feed_both_illumina_layouts() {
        let source = source_id("SRC1");
        let sample = sample_id("SMP1");
        let dna = library_id("LIB_DNA");
        let rna = library_id("LIB_RNA");
        let single_end_dna = SingleEndSequencingId::new("ACQ_SE_DNA").unwrap();
        let paired_end_dna = PairedEndSequencingId::new("ACQ_PE_DNA").unwrap();
        let single_end_rna = SingleEndSequencingId::new("ACQ_SE_RNA").unwrap();
        let paired_end_rna = PairedEndSequencingId::new("ACQ_PE_RNA").unwrap();

        let provenance = Provenance::new(
            [self::source("SRC1")],
            [self::sample("SMP1", [source])],
            [
                library("LIB_DNA", [sample.clone()], Input::FromDna),
                library(
                    "LIB_RNA",
                    [sample],
                    Input::FromRna {
                        strandedness: Strandedness::Forward,
                    },
                ),
            ],
            [
                Acquisition::IlluminaSingleEndSequencing(
                    SingleEndSequencing::new(
                        single_end_dna.clone(),
                        [dna.clone()],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
                Acquisition::IlluminaPairedEndSequencing(
                    PairedEndSequencing::new(
                        paired_end_dna.clone(),
                        [dna.clone()],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
                Acquisition::IlluminaSingleEndSequencing(
                    SingleEndSequencing::new(
                        single_end_rna.clone(),
                        [rna.clone()],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
                Acquisition::IlluminaPairedEndSequencing(
                    PairedEndSequencing::new(
                        paired_end_rna.clone(),
                        [rna.clone()],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        assert!(matches!(
            provenance.get(&dna),
            Some(Ok(library)) if matches!(library.input(), Input::FromDna)
        ));
        let dna_id = AnyLibraryId::from(dna.clone());
        assert!(matches!(
            provenance.get(&dna_id),
            Some(Ok(Library::P5P7(library))) if matches!(library.input(), Input::FromDna)
        ));
        let rna_id = LibraryIdRef::P5P7(&rna);
        assert!(matches!(
            provenance.get(&rna_id),
            Some(Ok(Library::P5P7(library)))
                if matches!(library.input(), Input::FromRna { .. })
        ));
        let single_end_dna_id = AcquisitionId::from(single_end_dna.clone());
        assert!(matches!(
            provenance.get(&single_end_dna_id),
            Some(Ok(Acquisition::IlluminaSingleEndSequencing(_)))
        ));
        let paired_end_dna_id = AcquisitionIdRef::IlluminaPairedEndSequencing(&paired_end_dna);
        assert!(matches!(
            provenance.get(&paired_end_dna_id),
            Some(Ok(Acquisition::IlluminaPairedEndSequencing(_)))
        ));
        let single_end_rna_id = AcquisitionIdRef::IlluminaSingleEndSequencing(&single_end_rna);
        assert!(matches!(
            provenance.get(&single_end_rna_id),
            Some(Ok(Acquisition::IlluminaSingleEndSequencing(_)))
        ));
        let paired_end_rna_id = AcquisitionIdRef::IlluminaPairedEndSequencing(&paired_end_rna);
        assert!(matches!(
            provenance.get(&paired_end_rna_id),
            Some(Ok(Acquisition::IlluminaPairedEndSequencing(_)))
        ));

        let wrong_variant = AcquisitionId::from(
            PairedEndSequencingId::new(single_end_dna.as_untyped().as_str()).unwrap(),
        );
        assert!(matches!(provenance.get(&wrong_variant), Some(Err(_))));
    }

    #[test]
    fn rejects_an_unknown_acquisition_library() {
        let source = source_id("SRC1");
        let sample = sample_id("SMP1");

        assert!(
            Provenance::new(
                [self::source("SRC1")],
                [self::sample("SMP1", [source])],
                [library("LIB1", [sample], Input::FromDna)],
                [Acquisition::IlluminaSingleEndSequencing(
                    SingleEndSequencing::new(
                        SingleEndSequencingId::new("ACQ1").unwrap(),
                        [library_id("MISSING_LIBRARY")],
                        Default::default(),
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
                Vec::<Acquisition>::new(),
            )
            .is_err()
        );

        assert!(
            Provenance::new(
                [self::source("SRC1")],
                Vec::<Sample>::new(),
                [library(
                    "LIB1",
                    [sample_id("MISSING_SAMPLE")],
                    Input::FromDna,
                )],
                Vec::<Acquisition>::new(),
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
                [library("DUP", [sample_id("SMP1")], Input::FromDna)],
                Vec::<Acquisition>::new(),
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_repeated_ids_within_an_entity_type() {
        assert!(
            Provenance::new(
                [self::source("DUP"), self::source("DUP")],
                Vec::<Sample>::new(),
                Vec::<Library>::new(),
                Vec::<Acquisition>::new(),
            )
            .is_err()
        );
    }

    #[test]
    fn serializes_and_resolves_p5p7_acquisition_inputs() {
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
                        "type": "P5P7",
                        "id": "LIB1",
                        "samples": ["SMP1"],
                        "input": {
                            "type": "FromRna",
                            "strandedness": "Unknown"
                        },
                        "meta": {"selection": "poly-A"}
                    }
                ],
                "acquisitions": [
                    {
                        "type": "IlluminaPairedEndSequencing",
                        "id": "ACQ1",
                        "libraries": ["LIB1"]
                    }
                ]
            }"#,
        )
        .unwrap();

        let acquisition_id = PairedEndSequencingId::new("ACQ1").unwrap();
        let acquisition = provenance
            .get(&acquisition_id)
            .expect("serialized acquisition should be present")
            .expect("serialized acquisition should have the requested type");
        assert!(acquisition.libraries().contains(&library_id("LIB1")));

        assert_eq!(
            serde_json::from_str::<Provenance>(&serde_json::to_string(&provenance).unwrap())
                .unwrap(),
            provenance
        );
    }
}
