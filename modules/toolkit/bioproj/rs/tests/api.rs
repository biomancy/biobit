use biobit_bioproj_rs::data::asset::Asset;
use biobit_bioproj_rs::data::asset::fastq as asset;
use biobit_bioproj_rs::data::dataset::fastq as dataset;
use biobit_bioproj_rs::data::{Data, Dataset, DatasetId};
use biobit_bioproj_rs::design::{
    Design, GroupContrast, GroupContrastId, MatchedPair, MatchedPairs, MatchedPairsId, Unit,
    UnitContrast, UnitContrastId, UnitId, UnitSet, UnitSetId,
};
use biobit_bioproj_rs::lookup;
use biobit_bioproj_rs::provenance::acquisition::illumina::{
    PairedEndSequencing, PairedEndSequencingId, SingleEndSequencing, SingleEndSequencingId,
};
use biobit_bioproj_rs::provenance::library::p5p7;
use biobit_bioproj_rs::provenance::{
    Acquisition, AcquisitionId, Library, Provenance, Sample, SampleId, Source, SourceId,
};
use biobit_bioproj_rs::{
    DesignId, Designs, Error, FilePath, Meta, MetaVal, NonEmpty, Project, ProjectId, Strandedness,
    Text, UntypedId,
};
use std::collections::BTreeSet;
use std::convert::Infallible;

// One valid graph covering both read layouts, mixed origins, and every design variant.
fn stub_project() -> Project {
    let source = SourceId::new("SRC1").unwrap();
    let virus = SourceId::new("VIRUS").unwrap();
    let sample = SampleId::new("SMP1").unwrap();
    let control_sample = SampleId::new("CONTROL").unwrap();

    let library = p5p7::LibraryId::new("LIB1").unwrap();
    let dna_library = p5p7::LibraryId::new("LIB_DNA").unwrap();
    let control_rna = p5p7::LibraryId::new("CONTROL_RNA").unwrap();

    let single = SingleEndSequencingId::new("ACQ_SE").unwrap();
    let paired = PairedEndSequencingId::new("ACQ_PE").unwrap();
    let paired_repeat = PairedEndSequencingId::new("ACQ_PE2").unwrap();

    let fastq = asset::FastqId::new("SINGLE").unwrap();
    let read1 = asset::FastqId::new("Z_READ1").unwrap();
    let read2 = asset::FastqId::new("A_READ2").unwrap();
    let repeat_read1 = asset::FastqId::new("RUN2_R1").unwrap();
    let repeat_read2 = asset::FastqId::new("RUN2_R2").unwrap();

    let unit_a = UnitId::new("UNIT_A").unwrap();
    let unit_b = UnitId::new("UNIT_B").unwrap();
    let unit_mixed = UnitId::new("UNIT_MIXED").unwrap();

    let sources = [
        Source::new(
            source.clone(),
            Some(Text::new("Homo sapiens").unwrap()),
            Default::default(),
            None,
        ),
        Source::new(
            virus.clone(),
            Some(Text::new("HSV-1").unwrap()),
            Default::default(),
            None,
        ),
    ];

    // The infected sample adds a viral source; the control has only the host.
    let samples = [
        Sample::new(
            sample.clone(),
            NonEmpty::try_from_iter([source.clone(), virus]).unwrap(),
            Meta::try_from_iter([(
                Text::new("Confluence").unwrap(),
                MetaVal::try_from("75%").unwrap(),
            )])
            .unwrap(),
            Some(Text::new("Infected cells").unwrap()),
        ),
        Sample::new(
            control_sample.clone(),
            NonEmpty::try_from_iter([source]).unwrap(),
            Default::default(),
            None,
        ),
    ];

    // Separate preparation paths can use the same collected material.
    let libraries = [
        Library::from(p5p7::Library::new(
            library.clone(),
            NonEmpty::try_from_iter([sample.clone()]).unwrap(),
            p5p7::Input::Rna {
                strandedness: Some(Strandedness::Reverse),
            },
            Meta::try_from_iter([(Text::new("selection:poly-A").unwrap(), MetaVal::from(true))])
                .unwrap(),
            None,
        )),
        Library::from(p5p7::Library::new(
            dna_library,
            NonEmpty::try_from_iter([sample]).unwrap(),
            p5p7::Input::Dna,
            Default::default(),
            None,
        )),
        Library::from(p5p7::Library::new(
            control_rna,
            NonEmpty::try_from_iter([control_sample]).unwrap(),
            p5p7::Input::Rna { strandedness: None },
            Default::default(),
            None,
        )),
    ];

    // Three measurements of the RNA library use two read layouts.
    let acquisitions = [
        Acquisition::from(SingleEndSequencing::new(
            single.clone(),
            NonEmpty::try_from_iter([library.clone()]).unwrap(),
            Default::default(),
            None,
        )),
        Acquisition::from(PairedEndSequencing::new(
            paired.clone(),
            NonEmpty::try_from_iter([library.clone()]).unwrap(),
            Meta::try_from_iter([(
                Text::new("machine").unwrap(),
                MetaVal::try_from("Illumina NovaSeq 6000").unwrap(),
            )])
            .unwrap(),
            Some(Text::new("First measurement").unwrap()),
        )),
        Acquisition::from(PairedEndSequencing::new(
            paired_repeat.clone(),
            NonEmpty::try_from_iter([library]).unwrap(),
            Default::default(),
            None,
        )),
    ];
    let provenance = Provenance::new(sources, samples, libraries, acquisitions).unwrap();

    // Paths are valid but need no files on disk for these API tests.
    let assets = [
        (fastq.clone(), "single.fq.gz"),
        (read1.clone(), "paired_R1.fq.gz"),
        (read2.clone(), "paired_R2.fq.gz"),
        (repeat_read1.clone(), "paired2_R1.fq.gz"),
        (repeat_read2.clone(), "paired2_R2.fq.gz"),
    ]
    .map(|(id, file)| {
        let location = FilePath::from_absolute(std::env::temp_dir().join(file)).unwrap();
        Asset::from(asset::Fastq::new(id, location, Default::default(), None))
    });

    let datasets = [
        Dataset::from(dataset::Single::new(
            dataset::SingleId::new("DATA_SE").unwrap(),
            single.clone(),
            NonEmpty::try_from_iter([fastq]).unwrap(),
            Default::default(),
            None,
        )),
        // Read-one sorts after read-two by ID; pairing keeps its own order.
        Dataset::from(
            dataset::Paired::new(
                dataset::PairedId::new("DATA_PE").unwrap(),
                paired.clone(),
                NonEmpty::try_from_iter([dataset::Pair::new(read1, read2).unwrap()]).unwrap(),
                Default::default(),
                None,
            )
            .unwrap(),
        ),
        Dataset::from(
            dataset::Paired::new(
                dataset::PairedId::new("DATA_PE2").unwrap(),
                paired_repeat.clone(),
                NonEmpty::try_from_iter([dataset::Pair::new(repeat_read1, repeat_read2).unwrap()])
                    .unwrap(),
                Default::default(),
                None,
            )
            .unwrap(),
        ),
    ];
    let data = Data::new(assets, datasets).unwrap();

    // Contrasts use disjoint units; UnitSet may include the overlapping pool.
    let units = [
        Unit::new(
            unit_a.clone(),
            NonEmpty::try_from_iter([AcquisitionId::from(single.clone())]).unwrap(),
            Default::default(),
            None,
        ),
        Unit::new(
            unit_b.clone(),
            NonEmpty::try_from_iter([AcquisitionId::from(paired.clone())]).unwrap(),
            Default::default(),
            None,
        ),
        Unit::new(
            unit_mixed.clone(),
            NonEmpty::try_from_iter([single.into(), paired.into(), paired_repeat.into()]).unwrap(),
            Meta::try_from_iter([(
                Text::new("title").unwrap(),
                MetaVal::try_from("Infected RNA").unwrap(),
            )])
            .unwrap(),
            None,
        ),
    ];
    let design_records = [
        Design::from(
            UnitContrast::new(
                UnitContrastId::new("DES_UNIT").unwrap(),
                Text::new("Control vs treatment").unwrap(),
                unit_a.clone(),
                unit_b.clone(),
                Default::default(),
                None,
            )
            .unwrap(),
        ),
        Design::from(
            GroupContrast::new(
                GroupContrastId::new("DES_GROUP").unwrap(),
                Text::new("Group contrast").unwrap(),
                NonEmpty::try_from_iter([unit_a.clone()]).unwrap(),
                NonEmpty::try_from_iter([unit_b.clone()]).unwrap(),
                Default::default(),
                None,
            )
            .unwrap(),
        ),
        Design::from(
            MatchedPairs::new(
                MatchedPairsId::new("DES_PAIRS").unwrap(),
                Text::new("Matched measurements").unwrap(),
                NonEmpty::try_from_iter(
                    [MatchedPair::new(unit_a.clone(), unit_b.clone()).unwrap()],
                )
                .unwrap(),
                Default::default(),
                None,
            )
            .unwrap(),
        ),
        Design::from(UnitSet::new(
            UnitSetId::new("DES_SET").unwrap(),
            Text::new("Joint analysis").unwrap(),
            NonEmpty::try_from_iter([unit_a, unit_b, unit_mixed]).unwrap(),
            Default::default(),
            None,
        )),
    ];
    let designs = Designs::new(units, design_records).unwrap();

    Project::new(
        ProjectId::new("API_STUB").unwrap(),
        provenance,
        data,
        designs,
        Default::default(),
        None,
    )
    .unwrap()
}

#[cfg(feature = "bitcode")]
#[test]
fn project_bitcode_roundtrip() {
    let project = stub_project();
    let restored = bitcode::decode::<Project>(&bitcode::encode(&project)).unwrap();
    assert_eq!(restored, project);
}

#[test]
fn lookups_preserve_the_precision_of_concrete_and_family_ids() {
    let project = stub_project();
    let provenance = project.provenance();
    let data = project.data();
    let designs = project.designs();

    // The explicit record types also check the public return types at compile time.
    macro_rules! check {
        ($domain:ident, $lookup:ident, $record:ty, $id:expr) => {{
            let id = $id;
            let record: &$record = $domain.get(&id).unwrap().unwrap();
            assert_eq!(record.id(), &id);
            let root = $domain.$lookup(id.as_untyped()).unwrap();
            assert_eq!($domain.get(&root.id()).unwrap().unwrap(), root);
            assert_eq!($domain.get(&root.id().to_owned()).unwrap().unwrap(), root);
        }};
    }

    check!(
        provenance,
        library,
        p5p7::Library,
        p5p7::LibraryId::new("LIB1").unwrap()
    );
    check!(
        provenance,
        acquisition,
        SingleEndSequencing,
        SingleEndSequencingId::new("ACQ_SE").unwrap()
    );
    check!(
        provenance,
        acquisition,
        PairedEndSequencing,
        PairedEndSequencingId::new("ACQ_PE").unwrap()
    );

    check!(
        data,
        asset,
        asset::Fastq,
        asset::FastqId::new("SINGLE").unwrap()
    );
    check!(
        data,
        dataset,
        dataset::Single,
        dataset::SingleId::new("DATA_SE").unwrap()
    );
    check!(
        data,
        dataset,
        dataset::Paired,
        dataset::PairedId::new("DATA_PE").unwrap()
    );

    check!(
        designs,
        design,
        UnitContrast,
        UnitContrastId::new("DES_UNIT").unwrap()
    );
    check!(
        designs,
        design,
        GroupContrast,
        GroupContrastId::new("DES_GROUP").unwrap()
    );
    check!(
        designs,
        design,
        MatchedPairs,
        MatchedPairsId::new("DES_PAIRS").unwrap()
    );
    check!(designs, design, UnitSet, UnitSetId::new("DES_SET").unwrap());

    // Direct IDs use the same optional result shape with an infallible check.
    let source: lookup::OptResult<&Source, Infallible> =
        provenance.get(&SourceId::new("SRC1").unwrap());
    let source = source.unwrap().unwrap();
    assert_eq!(provenance.source(source.id().as_untyped()), Some(source));
}

#[test]
fn lookup_distinguishes_a_missing_id_from_a_wrong_variant() {
    let project = stub_project();
    let provenance = project.provenance();
    let data = project.data();
    let designs = project.designs();

    // Concrete, owned family, and borrowed family keys agree on both outcomes.
    macro_rules! check {
        ($domain:ident, $concrete:ty, $family:ty, $local:literal, $expected:literal, $actual:literal) => {{
            let wrong = <$concrete>::new($local).unwrap();
            let family: $family = wrong.clone().into();
            for error in [
                $domain.get(&wrong).unwrap().unwrap_err(),
                $domain.get(&family).unwrap().unwrap_err(),
                $domain.get(&family.as_ref()).unwrap().unwrap_err(),
            ] {
                assert!(matches!(&error, Error::KindMismatch { id, expected, actual }
                    if id.as_str() == $local && expected == $expected && actual == $actual),
                    "{error:?}");
            }

            let missing = <$concrete>::new("MISSING").unwrap();
            let family: $family = missing.clone().into();
            assert!($domain.get(&missing).is_none());
            assert!($domain.get(&family).is_none());
            assert!($domain.get(&family.as_ref()).is_none());
        }};
    }

    check!(
        provenance,
        PairedEndSequencingId,
        AcquisitionId,
        "ACQ_SE",
        "IlluminaPairedEndSequencing",
        "IlluminaSingleEndSequencing"
    );
    check!(
        data,
        dataset::PairedId,
        DatasetId,
        "DATA_SE",
        "PairedFastq",
        "Fastq"
    );
    check!(
        designs,
        GroupContrastId,
        DesignId,
        "DES_UNIT",
        "GroupContrast",
        "UnitContrast"
    );

    // Untyped lookups report absence without claiming a variant.
    let missing = UntypedId::new("MISSING").unwrap();
    assert!(provenance.acquisition(&missing).is_none());
    assert!(data.dataset(&missing).is_none());
    assert!(designs.design(&missing).is_none());
    assert!(provenance.get(&SourceId::new("MISSING").unwrap()).is_none());
    assert!(provenance.get(&SampleId::new("MISSING").unwrap()).is_none());
    assert!(designs.get(&UnitId::new("MISSING").unwrap()).is_none());
}

#[test]
fn project_validates_standalone_domains_against_its_provenance() {
    let original = stub_project();
    let empty_provenance = Provenance::new([], [], [], []).unwrap();
    let empty_data = Data::new([], []).unwrap();
    let empty_designs = Designs::new([], []).unwrap();

    let data = Data::new(
        original.data().assets().cloned(),
        original.data().datasets().cloned(),
    )
    .unwrap();
    let designs = Designs::new(
        original.designs().units().cloned(),
        original.designs().designs().cloned(),
    )
    .unwrap();

    // Data is valid on its own, but its Dataset targets are absent here.
    let error = Project::new(
        original.id().clone(),
        empty_provenance.clone(),
        data,
        empty_designs,
        Default::default(),
        None,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        Error::MissingReference {
            owner: "Dataset",
            field: "acquisition",
            target: "Acquisition",
            ..
        }
    ));

    // Designs are also valid alone, but their Units need Provenance.
    let error = Project::new(
        original.id().clone(),
        empty_provenance,
        empty_data,
        designs,
        Default::default(),
        None,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        Error::MissingReference {
            owner: "Unit",
            field: "acquisitions",
            target: "Acquisition",
            ..
        }
    ));
}

#[test]
fn mixed_sources_and_library_preparations_remain_distinct() {
    let project = stub_project();
    let provenance = project.provenance();

    // The infected sample records both origins and its own condition.
    let sample = provenance
        .get(&SampleId::new("SMP1").unwrap())
        .unwrap()
        .unwrap();
    let organisms: BTreeSet<_> = sample
        .sources()
        .iter()
        .map(|id| {
            provenance
                .get(id)
                .unwrap()
                .unwrap()
                .organism()
                .unwrap()
                .as_str()
        })
        .collect();
    assert_eq!(organisms, BTreeSet::from(["Homo sapiens", "HSV-1"]));
    assert_eq!(sample.description().unwrap().as_str(), "Infected cells");
    assert_eq!(
        sample.meta().get("Confluence"),
        Some(&MetaVal::try_from("75%").unwrap())
    );

    // The control shares the host but not the infection source.
    let control = provenance
        .get(&SampleId::new("CONTROL").unwrap())
        .unwrap()
        .unwrap();
    assert!(control.sources().is_subset(sample.sources()));
    assert_eq!(control.sources().len(), 1);

    // One collected sample can produce separate RNA and DNA preparations.
    let rna = provenance
        .get(&p5p7::LibraryId::new("LIB1").unwrap())
        .unwrap()
        .unwrap();
    let dna = provenance
        .get(&p5p7::LibraryId::new("LIB_DNA").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(rna.samples(), dna.samples());
    assert!(rna.samples().contains(sample.id()));
    assert_eq!(
        rna.input(),
        &p5p7::Input::Rna {
            strandedness: Some(Strandedness::Reverse)
        }
    );
    assert_eq!(dna.input(), &p5p7::Input::Dna);
    assert_eq!(
        rna.meta().get("selection:poly-A"),
        Some(&MetaVal::Bool(true))
    );

    // Unreported strand specificity is distinct from a reverse-stranded prep.
    let control_rna = provenance
        .get(&p5p7::LibraryId::new("CONTROL_RNA").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        control_rna.input(),
        &p5p7::Input::Rna { strandedness: None }
    );
}

#[test]
fn a_unit_preserves_repeated_measurements_and_their_file_layouts() {
    let project = stub_project();
    let provenance = project.provenance();

    let unit = project
        .designs()
        .get(&UnitId::new("UNIT_MIXED").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        unit.meta().get("title"),
        Some(&MetaVal::try_from("Infected RNA").unwrap())
    );
    assert_eq!(unit.acquisitions().len(), 3);

    // Follow each pooled Acquisition to its Dataset, retaining read-pair order.
    let mut observed = Vec::new();
    for id in unit.acquisitions() {
        let acquisition = provenance.get(id).unwrap().unwrap();
        assert!(
            acquisition
                .libraries()
                .contains(&p5p7::LibraryId::new("LIB1").unwrap())
        );

        let datasets: Vec<_> = project
            .data()
            .datasets()
            .filter(|dataset| dataset.acquisition() == id.as_ref())
            .collect();
        assert_eq!(datasets.len(), 1);
        let assets: Vec<_> = match datasets[0] {
            Dataset::Fastq(value) => value.assets().iter().collect(),
            Dataset::PairedFastq(value) => value
                .pairs()
                .iter()
                .flat_map(|pair| [pair.read1(), pair.read2()])
                .collect(),
        };

        let files: Vec<_> = assets
            .into_iter()
            .map(|asset| {
                project
                    .data()
                    .get(asset)
                    .unwrap()
                    .unwrap()
                    .location()
                    .as_path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            })
            .collect();
        observed.push((id.as_untyped().as_str(), files));
    }
    observed.sort();
    assert_eq!(
        observed,
        vec![
            ("ACQ_PE", vec!["paired_R1.fq.gz", "paired_R2.fq.gz"]),
            ("ACQ_PE2", vec!["paired2_R1.fq.gz", "paired2_R2.fq.gz"]),
            ("ACQ_SE", vec!["single.fq.gz"]),
        ]
    );

    // Per-measurement annotations stay on their Acquisition.
    let paired = provenance
        .get(&PairedEndSequencingId::new("ACQ_PE").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        paired.meta().get("machine"),
        Some(&MetaVal::try_from("Illumina NovaSeq 6000").unwrap())
    );
    assert_eq!(paired.description().unwrap().as_str(), "First measurement");
}
