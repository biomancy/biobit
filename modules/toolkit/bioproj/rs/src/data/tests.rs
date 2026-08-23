use super::asset::fastq::{Fastq, FastqId};
use super::dataset::fastq::{Paired, PairedId, PairedInput, Single, SingleId, SingleInput};
use super::{Asset, AssetId, Data, Dataset, DatasetId};
use crate::provenance::acquisition::illumina::{
    PairedEndSequencing, PairedEndSequencingId, SingleEndSequencing, SingleEndSequencingId,
};
use crate::provenance::library::p5p7;
use crate::provenance::{Acquisition, Library, Provenance, Sample, SampleId, Source, SourceId};

fn provenance(single_ids: &[&str], paired_ids: &[&str]) -> Provenance {
    let source_id = SourceId::new("SRC1").unwrap();
    let sample_id = SampleId::new("SMP1").unwrap();
    let library_id = p5p7::LibraryId::new("LIB1").unwrap();
    Provenance::new(
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
        single_ids
            .iter()
            .map(|id| {
                Acquisition::IlluminaSingleEndSequencing(
                    SingleEndSequencing::new(
                        SingleEndSequencingId::new(*id).unwrap(),
                        [library_id.clone()],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                )
            })
            .chain(paired_ids.iter().map(|id| {
                Acquisition::IlluminaPairedEndSequencing(
                    PairedEndSequencing::new(
                        PairedEndSequencingId::new(*id).unwrap(),
                        [library_id.clone()],
                        Default::default(),
                        None::<String>,
                    )
                    .unwrap(),
                )
            })),
    )
    .unwrap()
}

fn assets(ids: &[&str]) -> Vec<Asset> {
    ids.iter()
        .map(|id| {
            Asset::Fastq(
                Fastq::new(
                    FastqId::new(*id).unwrap(),
                    format!("file:{id}.fq.gz"),
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            )
        })
        .collect()
}

fn single_input(id: &str) -> SingleInput {
    SingleInput::IlluminaSingleEndSequencing(SingleEndSequencingId::new(id).unwrap())
}

fn paired_input(id: &str) -> PairedInput {
    PairedInput::IlluminaPairedEndSequencing(PairedEndSequencingId::new(id).unwrap())
}

#[test]
fn serializes_and_resolves_assets_with_their_datasets() {
    let provenance = provenance(&["ACQ_SINGLE"], &["ACQ_PAIRED"]);
    let data = Data::new(
        &provenance,
        assets(&["R1", "R2", "LONG"]),
        [
            Dataset::Fastq(
                Single::new(
                    SingleId::new("DATA_SINGLE").unwrap(),
                    single_input("ACQ_SINGLE"),
                    [FastqId::new("LONG").unwrap()],
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            ),
            Dataset::PairedFastq(
                Paired::new(
                    PairedId::new("DATA_PAIRED").unwrap(),
                    paired_input("ACQ_PAIRED"),
                    [(FastqId::new("R1").unwrap(), FastqId::new("R2").unwrap())],
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            ),
        ],
    )
    .unwrap();

    let asset_id = AssetId::from(FastqId::new("LONG").unwrap());
    assert!(matches!(
        data.asset(asset_id.as_ref()),
        Some(Asset::Fastq(_))
    ));

    let dataset_id = DatasetId::from(SingleId::new("DATA_SINGLE").unwrap());
    assert!(matches!(
        data.dataset(dataset_id.as_ref()),
        Some(Dataset::Fastq(_))
    ));

    let wrong_variant = DatasetId::from(PairedId::new("DATA_SINGLE").unwrap());
    assert!(data.dataset(wrong_variant.as_ref()).is_none());

    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(
        json,
        r#"{"assets":[{"type":"Fastq","id":"LONG","location":"file:LONG.fq.gz"},{"type":"Fastq","id":"R1","location":"file:R1.fq.gz"},{"type":"Fastq","id":"R2","location":"file:R2.fq.gz"}],"datasets":[{"type":"PairedFastq","id":"DATA_PAIRED","acquisition":"ACQ_PAIRED","pairs":[["R1","R2"]]},{"type":"Fastq","id":"DATA_SINGLE","acquisition":"ACQ_SINGLE","assets":["LONG"]}]}"#
    );

    let mut deserializer = serde_json::Deserializer::from_str(&json);
    let resolved = Data::deserialize_with_provenance(&provenance, &mut deserializer).unwrap();
    assert_eq!(resolved, data);
}

#[test]
fn rejects_a_dataset_layout_incompatible_with_its_acquisition() {
    let provenance = provenance(&["ACQ_SINGLE"], &[]);
    let mut deserializer = serde_json::Deserializer::from_str(
        r#"{
            "assets": [
                {"type": "Fastq", "id": "R1", "location": "file:R1.fq.gz"},
                {"type": "Fastq", "id": "R2", "location": "file:R2.fq.gz"}
            ],
            "datasets": [{
                "type": "PairedFastq",
                "id": "DATA1",
                "acquisition": "ACQ_SINGLE",
                "pairs": [["R1", "R2"]]
            }]
        }"#,
    );

    assert!(Data::deserialize_with_provenance(&provenance, &mut deserializer).is_err());
}

#[test]
fn permits_overlapping_datasets_for_one_acquisition() {
    let provenance = provenance(&["ACQ1"], &[]);
    Data::new(
        &provenance,
        assets(&["LANE1", "LANE2"]),
        [
            Dataset::Fastq(
                Single::new(
                    SingleId::new("RAW").unwrap(),
                    single_input("ACQ1"),
                    [
                        FastqId::new("LANE1").unwrap(),
                        FastqId::new("LANE2").unwrap(),
                    ],
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            ),
            Dataset::Fastq(
                Single::new(
                    SingleId::new("QC").unwrap(),
                    single_input("ACQ1"),
                    [FastqId::new("LANE1").unwrap()],
                    Default::default(),
                    Some("lane 2 failed QC"),
                )
                .unwrap(),
            ),
        ],
    )
    .unwrap();
}

#[test]
fn rejects_cross_acquisition_asset_reuse() {
    let provenance = provenance(&["ACQ1", "ACQ2"], &[]);
    let result = Data::new(
        &provenance,
        assets(&["SHARED"]),
        [
            Dataset::Fastq(
                Single::new(
                    SingleId::new("DATA1").unwrap(),
                    single_input("ACQ1"),
                    [FastqId::new("SHARED").unwrap()],
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            ),
            Dataset::Fastq(
                Single::new(
                    SingleId::new("DATA2").unwrap(),
                    single_input("ACQ2"),
                    [FastqId::new("SHARED").unwrap()],
                    Default::default(),
                    None::<String>,
                )
                .unwrap(),
            ),
        ],
    );
    assert!(result.is_err());
}

#[test]
fn paired_layout_uses_each_asset_once() {
    assert!(
        Paired::new(
            PairedId::new("DATA").unwrap(),
            paired_input("ACQ1"),
            [
                (FastqId::new("A").unwrap(), FastqId::new("B").unwrap()),
                (FastqId::new("A").unwrap(), FastqId::new("C").unwrap()),
            ],
            Default::default(),
            None::<String>,
        )
        .is_err()
    );
}
