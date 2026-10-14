//! JSON 0.0.1 wire representations for the provenance domain.
//!
//! Loading builds domain entities and lets [`Provenance`] validate their
//! relationships. Saving borrows the validated domain values.

use crate::provenance::acquisition::illumina::{
    PairedEndSequencing, PairedEndSequencingId, SingleEndSequencing, SingleEndSequencingId,
};
use crate::provenance::library::p5p7;
use crate::provenance::{Acquisition, Library, Provenance, Sample, SampleId, Source, SourceId};
use crate::{Meta, NonEmpty, OrderedSet, Result, Strandedness, Text};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceFromWire {
    id: SourceId,
    #[serde(default)]
    organism: Option<Text>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<Text>,
}

impl SourceFromWire {
    fn resolve(self) -> Source {
        Source::new(self.id, self.organism, self.meta, self.description)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SampleFromWire {
    id: SampleId,
    sources: NonEmpty<OrderedSet<SourceId>>,
    #[serde(default)]
    meta: Meta,
    #[serde(default)]
    description: Option<Text>,
}

impl SampleFromWire {
    fn resolve(self) -> Sample {
        Sample::new(self.id, self.sources, self.meta, self.description)
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "UPPERCASE", deny_unknown_fields)]
enum InputFromWire {
    Dna,
    Rna {
        #[serde(default, deserialize_with = "deserialize_strandedness")]
        strandedness: Option<Strandedness>,
    },
}

// Only omission defaults to None; present values must name a strandedness.
fn deserialize_strandedness<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Strandedness>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Strandedness>::deserialize(deserializer)? {
        Some(value) => Ok(Some(value)),
        None => Err(serde::de::Error::custom(
            "Strandedness must not be null; omit the field if it was not reported, \
             or specify Forward, Reverse, or Unstranded",
        )),
    }
}

impl InputFromWire {
    fn resolve(self) -> p5p7::Input {
        match self {
            Self::Dna => p5p7::Input::Dna,
            Self::Rna { strandedness } => p5p7::Input::Rna { strandedness },
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum LibraryFromWire {
    P5P7 {
        id: p5p7::LibraryId,
        samples: NonEmpty<OrderedSet<SampleId>>,
        input: InputFromWire,
        #[serde(default)]
        meta: Meta,
        #[serde(default)]
        description: Option<Text>,
    },
}

impl LibraryFromWire {
    fn resolve(self) -> Library {
        match self {
            Self::P5P7 {
                id,
                samples,
                input,
                meta,
                description,
            } => p5p7::Library::new(id, samples, input.resolve(), meta, description).into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum AcquisitionFromWire {
    IlluminaSingleEndSequencing {
        id: SingleEndSequencingId,
        libraries: NonEmpty<OrderedSet<p5p7::LibraryId>>,
        #[serde(default)]
        meta: Meta,
        #[serde(default)]
        description: Option<Text>,
    },
    IlluminaPairedEndSequencing {
        id: PairedEndSequencingId,
        libraries: NonEmpty<OrderedSet<p5p7::LibraryId>>,
        #[serde(default)]
        meta: Meta,
        #[serde(default)]
        description: Option<Text>,
    },
}

impl AcquisitionFromWire {
    fn resolve(self) -> Acquisition {
        match self {
            Self::IlluminaSingleEndSequencing {
                id,
                libraries,
                meta,
                description,
            } => SingleEndSequencing::new(id, libraries, meta, description).into(),
            Self::IlluminaPairedEndSequencing {
                id,
                libraries,
                meta,
                description,
            } => PairedEndSequencing::new(id, libraries, meta, description).into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProvenanceFromWire {
    sources: Vec<SourceFromWire>,
    samples: Vec<SampleFromWire>,
    libraries: Vec<LibraryFromWire>,
    acquisitions: Vec<AcquisitionFromWire>,
}

impl ProvenanceFromWire {
    pub(super) fn resolve(self) -> Result<Provenance> {
        Provenance::new(
            self.sources.into_iter().map(SourceFromWire::resolve),
            self.samples.into_iter().map(SampleFromWire::resolve),
            self.libraries.into_iter().map(LibraryFromWire::resolve),
            self.acquisitions
                .into_iter()
                .map(AcquisitionFromWire::resolve),
        )
    }
}

#[derive(Serialize)]
struct SourceToWire<'a> {
    id: &'a SourceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    organism: Option<&'a Text>,
    #[serde(skip_serializing_if = "Meta::is_empty")]
    meta: &'a Meta,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a Text>,
}

impl<'a> SourceToWire<'a> {
    fn new(source: &'a Source) -> Self {
        Self {
            id: source.id(),
            organism: source.organism(),
            meta: source.meta(),
            description: source.description(),
        }
    }
}

#[derive(Serialize)]
struct SampleToWire<'a> {
    id: &'a SampleId,
    sources: &'a NonEmpty<OrderedSet<SourceId>>,
    #[serde(skip_serializing_if = "Meta::is_empty")]
    meta: &'a Meta,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a Text>,
}

impl<'a> SampleToWire<'a> {
    fn new(sample: &'a Sample) -> Self {
        Self {
            id: sample.id(),
            sources: sample.sources(),
            meta: sample.meta(),
            description: sample.description(),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
enum InputToWire<'a> {
    Dna,
    Rna {
        #[serde(skip_serializing_if = "Option::is_none")]
        strandedness: Option<&'a Strandedness>,
    },
}

impl<'a> InputToWire<'a> {
    fn new(input: &'a p5p7::Input) -> Self {
        match input {
            p5p7::Input::Dna => Self::Dna,
            p5p7::Input::Rna { strandedness } => Self::Rna {
                strandedness: strandedness.as_ref(),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum LibraryToWire<'a> {
    P5P7 {
        id: &'a p5p7::LibraryId,
        samples: &'a NonEmpty<OrderedSet<SampleId>>,
        input: InputToWire<'a>,
        #[serde(skip_serializing_if = "Meta::is_empty")]
        meta: &'a Meta,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a Text>,
    },
}

impl<'a> LibraryToWire<'a> {
    fn new(library: &'a Library) -> Self {
        match library {
            Library::P5P7(library) => Self::P5P7 {
                id: library.id(),
                samples: library.samples(),
                input: InputToWire::new(library.input()),
                meta: library.meta(),
                description: library.description(),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum AcquisitionToWire<'a> {
    IlluminaSingleEndSequencing {
        id: &'a SingleEndSequencingId,
        libraries: &'a NonEmpty<OrderedSet<p5p7::LibraryId>>,
        #[serde(skip_serializing_if = "Meta::is_empty")]
        meta: &'a Meta,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a Text>,
    },
    IlluminaPairedEndSequencing {
        id: &'a PairedEndSequencingId,
        libraries: &'a NonEmpty<OrderedSet<p5p7::LibraryId>>,
        #[serde(skip_serializing_if = "Meta::is_empty")]
        meta: &'a Meta,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<&'a Text>,
    },
}

impl<'a> AcquisitionToWire<'a> {
    fn new(acquisition: &'a Acquisition) -> Self {
        match acquisition {
            Acquisition::IlluminaSingleEndSequencing(acquisition) => {
                Self::IlluminaSingleEndSequencing {
                    id: acquisition.id(),
                    libraries: acquisition.libraries(),
                    meta: acquisition.meta(),
                    description: acquisition.description(),
                }
            }
            Acquisition::IlluminaPairedEndSequencing(acquisition) => {
                Self::IlluminaPairedEndSequencing {
                    id: acquisition.id(),
                    libraries: acquisition.libraries(),
                    meta: acquisition.meta(),
                    description: acquisition.description(),
                }
            }
        }
    }
}

#[derive(Serialize)]
pub(super) struct ProvenanceToWire<'a> {
    sources: Vec<SourceToWire<'a>>,
    samples: Vec<SampleToWire<'a>>,
    libraries: Vec<LibraryToWire<'a>>,
    acquisitions: Vec<AcquisitionToWire<'a>>,
}

impl<'a> ProvenanceToWire<'a> {
    pub(super) fn new(provenance: &'a Provenance) -> Self {
        Self {
            sources: provenance.sources().map(SourceToWire::new).collect(),
            samples: provenance.samples().map(SampleToWire::new).collect(),
            libraries: provenance.libraries().map(LibraryToWire::new).collect(),
            acquisitions: provenance
                .acquisitions()
                .map(AcquisitionToWire::new)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AcquisitionFromWire, AcquisitionToWire, InputFromWire, InputToWire, LibraryFromWire,
        LibraryToWire, SourceFromWire, SourceToWire,
    };
    use crate::provenance::acquisition::illumina::{
        PairedEndSequencing, PairedEndSequencingId, SingleEndSequencing, SingleEndSequencingId,
    };
    use crate::provenance::library::p5p7;
    use crate::provenance::{Acquisition, Library, SampleId, Source, SourceId};
    use crate::{NonEmpty, Strandedness};

    #[test]
    fn omits_absent_source_annotations() {
        let source = Source::new(
            SourceId::new("SRC1").unwrap(),
            None,
            Default::default(),
            None,
        );

        let json = serde_json::to_string(&SourceToWire::new(&source)).unwrap();
        assert_eq!(json, r#"{"id":"SRC1"}"#);
        let from_wire: SourceFromWire = serde_json::from_str(&json).unwrap();
        assert_eq!(from_wire.resolve(), source);
    }

    #[test]
    fn round_trips_flat_tagged_library_variants() {
        let library: Library = p5p7::Library::new(
            p5p7::LibraryId::new("LIB1").unwrap(),
            NonEmpty::try_from_iter([SampleId::new("SMP1").unwrap()]).unwrap(),
            p5p7::Input::Rna {
                strandedness: Some(Strandedness::Forward),
            },
            Default::default(),
            None,
        )
        .into();

        let json = serde_json::to_string(&LibraryToWire::new(&library)).unwrap();
        assert_eq!(
            json,
            r#"{"type":"P5P7","id":"LIB1","samples":["SMP1"],"input":{"type":"RNA","strandedness":"Forward"}}"#
        );
        let from_wire: LibraryFromWire = serde_json::from_str(&json).unwrap();
        assert_eq!(from_wire.resolve(), library);
    }

    #[test]
    fn round_trips_rna_input_without_strandedness() {
        let input: InputFromWire = serde_json::from_str(r#"{"type":"RNA"}"#).unwrap();
        let input = input.resolve();
        assert_eq!(input, p5p7::Input::Rna { strandedness: None });
        assert_eq!(
            serde_json::to_string(&InputToWire::new(&input)).unwrap(),
            r#"{"type":"RNA"}"#
        );
    }

    #[test]
    fn rna_input_rejects_null_strandedness() {
        let error = serde_json::from_str::<InputFromWire>(r#"{"type":"RNA","strandedness":null}"#)
            .err()
            .expect("null strandedness must be rejected");
        let message = error.to_string();
        assert!(message.contains("Strandedness must not be null"));
        assert!(message.contains("omit the field"));
    }

    #[test]
    fn round_trips_illumina_acquisition_variants() {
        let library = p5p7::LibraryId::new("LIB1").unwrap();
        let acquisitions = [
            (
                Acquisition::from(SingleEndSequencing::new(
                    SingleEndSequencingId::new("SINGLE").unwrap(),
                    NonEmpty::try_from_iter([library.clone()]).unwrap(),
                    Default::default(),
                    None,
                )),
                r#"{"type":"IlluminaSingleEndSequencing","id":"SINGLE","libraries":["LIB1"]}"#,
            ),
            (
                Acquisition::from(PairedEndSequencing::new(
                    PairedEndSequencingId::new("PAIRED").unwrap(),
                    NonEmpty::try_from_iter([library]).unwrap(),
                    Default::default(),
                    None,
                )),
                r#"{"type":"IlluminaPairedEndSequencing","id":"PAIRED","libraries":["LIB1"]}"#,
            ),
        ];

        for (acquisition, expected) in acquisitions {
            let json = serde_json::to_string(&AcquisitionToWire::new(&acquisition)).unwrap();
            assert_eq!(json, expected);
            let from_wire: AcquisitionFromWire = serde_json::from_str(&json).unwrap();
            assert_eq!(from_wire.resolve(), acquisition);
        }
    }
}
