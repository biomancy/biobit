mod utils;

use biobit_bioproj_rs::io::json;
use biobit_bioproj_rs::{Error, Project};
use std::error::Error as _;
use std::fs;
use utils::Workspace;

const PROJECT: &[u8] = include_bytes!("../../resources/valid/all_variants.json");

fn project(workspace: &Workspace) -> Project {
    let path = workspace.join("bioproj.json");
    fs::write(&path, PROJECT).unwrap();
    json::read(path).unwrap()
}

#[test]
fn file_roundtrips_preserve_the_complete_project() {
    let workspace = Workspace::new();
    let project = project(&workspace);

    let path = workspace.join("bioproj.json");
    json::write(&path, &project, true).unwrap();

    let restored = json::read(&path).unwrap();
    assert_eq!(restored, project);

    // Rewriting the same project is byte-stable and leaves no staging file.
    let original = fs::read(&path).unwrap();
    json::write(&path, &restored, true).unwrap();
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(fs::read_dir(workspace.path()).unwrap().count(), 1);

    for pretty in [false, true] {
        json::write(&path, &restored, pretty).unwrap();
        assert_eq!(fs::read(&path).unwrap().starts_with(b"{\n"), pretty);
        assert_eq!(json::read(&path).unwrap(), restored);
    }
}

#[test]
fn saving_in_another_directory_preserves_asset_targets() {
    let workspace = Workspace::new();
    let directory = fs::canonicalize(workspace.path()).unwrap();

    let source = directory.join("source");
    fs::create_dir(&source).unwrap();
    let source_manifest = source.join("bioproj.json");
    fs::write(&source_manifest, PROJECT).unwrap();
    let project = json::read(&source_manifest).unwrap();
    fs::write(source.join("DNA.fq.gz"), []).unwrap();

    let destination = directory.join("destination");
    fs::create_dir(&destination).unwrap();
    let destination_manifest = destination.join("bioproj.json");
    json::write(&destination_manifest, &project, true).unwrap();

    let restored = json::read(&destination_manifest).unwrap();
    let dna = restored.data().asset(&"DNA".parse().unwrap()).unwrap();
    assert!(!dna.location().is_absolute());
    assert_eq!(
        dna.location().as_path(),
        std::path::Path::new("../source/DNA.fq.gz")
    );
    assert_eq!(
        fs::canonicalize(dna.location().absolute()).unwrap(),
        fs::canonicalize(source.join("DNA.fq.gz")).unwrap()
    );
}

#[test]
fn absolute_asset_locations_stay_absolute_when_saving_elsewhere() {
    let workspace = Workspace::new();
    let source = workspace.join("source.json");
    let absolute = workspace.join("DNA.fq.gz");
    let mut document: serde_json::Value = serde_json::from_slice(PROJECT).unwrap();
    document["data"]["assets"][0]["location"] = absolute.to_str().unwrap().into();
    fs::write(&source, serde_json::to_vec(&document).unwrap()).unwrap();

    let destination = workspace.join("other");
    fs::create_dir(&destination).unwrap();
    let project = json::read(&source).unwrap();
    json::write(destination.join("copy.json"), &project, true).unwrap();

    let saved = json::read(destination.join("copy.json")).unwrap();
    let asset = saved.data().asset(&"DNA".parse().unwrap()).unwrap();
    assert_eq!(asset.location().as_path(), absolute);
    assert_eq!(asset.location().absolute(), absolute);
}

#[cfg(unix)]
#[test]
fn saving_through_a_directory_symlink_uses_its_physical_root() {
    use std::os::unix::fs::symlink;
    let workspace = Workspace::new();

    let source = workspace.join("source");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("DNA.fq.gz"), []).unwrap();
    let source_manifest = source.join("bioproj.json");
    fs::write(&source_manifest, PROJECT).unwrap();

    // The link sits at a different depth from its physical destination.
    let destination = workspace.join("physical/nested/destination");
    fs::create_dir_all(&destination).unwrap();
    let linked_destination = workspace.join("destination-link");
    symlink(&destination, &linked_destination).unwrap();
    let output = linked_destination.join("bioproj.json");

    let project = json::read(&source_manifest).unwrap();
    json::write(&output, &project, true).unwrap();
    let restored = json::read(&output).unwrap();

    // Rebasing must use the physical directory reached through the link.
    let asset = restored.data().asset(&"DNA".parse().unwrap()).unwrap();
    let expected = std::path::Path::new("../../../source/DNA.fq.gz");
    assert!(!asset.location().is_absolute());
    assert_eq!(asset.location().as_path(), expected);
    assert_eq!(
        asset.location().absolute(),
        fs::canonicalize(&destination).unwrap().join(expected)
    );
    assert_eq!(
        fs::canonicalize(asset.location().absolute()).unwrap(),
        fs::canonicalize(source.join("DNA.fq.gz")).unwrap()
    );

    // Saving again must not change the relative location.
    json::write(&output, &restored, true).unwrap();
    let saved_again = json::read(&output).unwrap();
    let asset = saved_again.data().asset(&"DNA".parse().unwrap()).unwrap();
    assert_eq!(asset.location().as_path(), expected);
}

#[test]
fn failed_publication_preserves_the_destination_and_cleans_up() {
    let workspace = Workspace::new();
    let destination = workspace.join("existing-directory");
    fs::create_dir(&destination).unwrap();
    fs::write(destination.join("marker"), b"keep").unwrap();

    let project = project(&workspace);
    assert!(json::write(&destination, &project, true).is_err());
    assert_eq!(fs::read(destination.join("marker")).unwrap(), b"keep");
    assert_eq!(fs::read_dir(workspace.path()).unwrap().count(), 2);
}

#[test]
fn io_and_json_failures_preserve_their_sources() {
    let workspace = Workspace::new();
    let missing = workspace.join("missing.json");
    let error = json::read(&missing).unwrap_err();
    assert!(matches!(&error, Error::Io { path: Some(path), source, .. }
        if path == &missing && source.kind() == std::io::ErrorKind::NotFound));
    assert!(error.source().unwrap().is::<std::io::Error>());

    let malformed = workspace.join("malformed.json");
    fs::write(&malformed, b"{").unwrap();
    let error = json::read(&malformed).unwrap_err();
    assert!(matches!(&error, Error::Json { source, .. } if source.is_eof()));
    assert!(error.source().unwrap().is::<serde_json::Error>());
}

mod validation {
    use super::{json, utils::Workspace};
    use biobit_bioproj_rs::Error;
    use serde_json::{Value, json as value};
    use std::fs;

    // Compare public diagnostic fields, not Display wording or Serde line numbers.
    fn diagnostic(error: &Error) -> Value {
        match error {
            Error::MissingReference {
                owner,
                id,
                field,
                target,
                target_id,
            } => value!({
                "error": "MissingReference", "owner": owner, "id": id,
                "field": field, "target": target, "target_id": target_id,
            }),
            Error::DuplicateKey { collection, key } => value!({
                "error": "DuplicateKey", "collection": collection, "key": key,
            }),
            Error::InvalidValue { field, reason } => value!({
                "error": "InvalidValue", "field": field, "reason": reason,
            }),
            Error::KindMismatch {
                id,
                expected,
                actual,
            } => value!({
                "error": "KindMismatch", "id": id, "expected": expected, "actual": actual,
            }),
            Error::AcquisitionOverlap {
                design,
                acquisition,
                first_unit,
                second_unit,
            } => value!({
                "error": "AcquisitionOverlap", "design": design, "acquisition": acquisition,
                "first_unit": first_unit, "second_unit": second_unit,
            }),
            Error::DatasetSubset {
                acquisition,
                subset,
                superset,
            } => value!({
                "error": "DatasetSubset", "acquisition": acquisition,
                "subset": subset, "superset": superset,
            }),
            Error::AssetOwnership {
                asset,
                first_acquisition,
                second_acquisition,
            } => value!({
                "error": "AssetOwnership", "asset": asset,
                "first_acquisition": first_acquisition, "second_acquisition": second_acquisition,
            }),
            Error::UnrepresentedAcquisition { id } => value!({
                "error": "UnrepresentedAcquisition", "id": id,
            }),
            Error::UnusedAsset { id } => value!({"error": "UnusedAsset", "id": id}),
            Error::UnsupportedSchema { found, supported } => value!({
                "error": "UnsupportedSchema", "found": found, "supported": supported,
            }),
            Error::Json { operation, source } => {
                let category = match source.classify() {
                    serde_json::error::Category::Io => "io",
                    serde_json::error::Category::Syntax => "syntax",
                    serde_json::error::Category::Data => "data",
                    serde_json::error::Category::Eof => "eof",
                };
                value!({"error": "Json", "operation": operation, "category": category})
            }
            other => panic!("unexpected fixture error: {other:?}"),
        }
    }

    fn assert_valid(input: &str) {
        let workspace = Workspace::new();
        let path = workspace.join("bioproj.json");
        fs::write(&path, input).unwrap();
        json::read(path).unwrap();
    }

    macro_rules! valid_fixtures {
        ($($name:ident),+ $(,)?) => {
            $(
                #[test]
                fn $name() {
                    assert_valid(include_str!(concat!("../../resources/valid/", stringify!($name), ".json")));
                }
            )+
        };
    }

    valid_fixtures!(
        empty_domains,
        shared_local_ids,
        all_variants,
        equivalent_datasets,
        sequencing_project,
    );

    fn assert_invalid(input: &str, expected: &str) {
        // Read raw text: parsing into Value first would erase duplicate JSON keys.
        let workspace = Workspace::new();
        let path = workspace.join("bioproj.json");
        fs::write(&path, input).unwrap();
        let error = json::read(path).unwrap_err();
        let expected: Value = serde_json::from_str(expected).unwrap();
        assert_eq!(diagnostic(&error), expected, "{error:?}");
    }

    macro_rules! invalid_fixtures {
        ($($name:ident),+ $(,)?) => {
            $(
                #[test]
                fn $name() {
                    assert_invalid(
                        include_str!(concat!("../../resources/invalid/", stringify!($name), ".json")),
                        include_str!(concat!("../../resources/invalid/", stringify!($name), ".error.json")),
                    );
                }
            )+
        };
    }

    invalid_fixtures!(
        // Missing graph references.
        missing_source,
        missing_sample,
        missing_library,
        missing_dataset_acquisition,
        missing_single_asset,
        missing_paired_asset,
        missing_unit_acquisition,
        missing_design_unit,
        missing_contrast_unit,
        // Shared ID namespaces and incompatible record kinds.
        duplicate_source_id,
        duplicate_acquisition_variant_id,
        duplicate_dataset_variant_id,
        duplicate_design_variant_id,
        paired_dataset_with_single_acquisition,
        single_dataset_with_paired_acquisition,
        // Dataset coverage, ownership, and design independence.
        unrepresented_acquisition,
        unused_asset,
        asset_ownership,
        single_dataset_subset,
        paired_dataset_subset,
        unit_contrast_overlap,
        group_between_arms_overlap,
        matched_within_pair_overlap,
        // Schema dispatch and invalid JSON fields or values.
        unsupported_major_schema,
        missing_domain,
        unknown_field,
        invalid_organism,
        missing_design_name,
        blank_design_name,
        duplicate_parent_reference,
        reused_pair_asset,
        malformed_json,
        trailing_json,
        duplicate_schema_field,
        duplicate_project_meta_key,
    );
}
