use super::run::Run;
use crate::experiment::Experiment;
use crate::library::Library;
use crate::sample::Sample;
use crate::storage::Storage;
use biobit_core_rs::ngs;
use eyre::{Result, ensure};
use std::collections::BTreeSet;
use std::sync::Arc;

pub fn non_empty_collection<T>(name: &str, col: &[T]) -> Result<()> {
    ensure!(!col.is_empty(), "'{name}' must not be empty");
    Ok(())
}

pub fn unique_ids<'a>(id_type: &'static str, ids: impl Iterator<Item = &'a str>) -> Result<()> {
    let counter = ids
        .into_iter()
        .fold(std::collections::HashMap::new(), |mut acc, id| {
            *acc.entry(id).or_insert(0) += 1;
            acc
        });
    let mut errors = vec![];
    for (id, count) in counter {
        if count > 1 {
            errors.push(format!("'{}' is not unique (found {} times)", id, count));
        }
        if errors.len() >= 10 {
            errors.push("Other errors omitted...".to_string());
            break;
        }
    }
    ensure!(
        errors.is_empty(),
        "{id_type} uniqueness errors:\n{}",
        errors.join("\n")
    );
    Ok(())
}

pub fn layout_storage_compatibility(ind: &str, layout: &ngs::Layout, runs: &[Run]) -> Result<()> {
    let error = || eyre::eyre!("Incompatible layout '{layout:?}' for run(s) in ind '{ind}'");
    for run in runs {
        let storage = run.storage();
        match layout {
            ngs::Layout::Paired { .. } => match storage {
                Storage::PairedFastq { .. } => continue,
                _ => return Err(error()),
            },
            ngs::Layout::Single { .. } => match storage {
                Storage::SingleFastq { .. } => continue,
                _ => return Err(error()),
            },
        }
    }
    Ok(())
}

pub fn validate_graph_integrity(
    samples: &[Arc<Sample>],
    libraries: &[Arc<Library>],
    experiments: &[Arc<Experiment>],
) -> Result<()> {
    let valid_sample_ids: BTreeSet<&str> = samples.iter().map(|s| s.ind()).collect();
    let valid_library_ids: BTreeSet<&str> = libraries.iter().map(|l| l.ind()).collect();

    // Check that all libraries point to valid samples
    for lib in libraries {
        ensure!(
            valid_sample_ids.contains(lib.sample().ind()),
            "Dangling reference: Library '{}' points to unknown Sample '{}'",
            lib.ind(),
            lib.sample().ind()
        );
    }

    // Check that all experiments point to valid libraries
    for exp in experiments {
        ensure!(
            valid_library_ids.contains(exp.library().ind()),
            "Dangling reference: Experiment '{}' points to unknown Library '{}'",
            exp.ind(),
            exp.library().ind()
        );
    }

    Ok(())
}
