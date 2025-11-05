use super::experiment::{DeserializedExperiment, Experiment};
use super::library::{DeserializedLibrary, Library};
use super::sample::Sample;
use super::{parse, validate};
use eyre::Result;
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug, Display, Formatter};
use std::sync::Arc;

/// Represents a top-level sequencing project.
///
/// A `Project` is the root container for all other entities.
///
/// The `Project`'s constructor is responsible for performing a set of validations:
/// *  Uniqueness of all entity IDs (`Sample`, `Library`, `Experiment`, `Run`).
/// *  Graph integrity: ensuring no "dangling references" exist
///    (e.g., an `Experiment` pointing to a `Library` that isn't
///    in the project, or a `Library` pointing to a `Sample` that
///    isn't in the project).
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct Project {
    ind: String,
    samples: Vec<Arc<Sample>>,
    libraries: Vec<Arc<Library>>,
    experiments: Vec<Arc<Experiment>>,
    meta: BTreeMap<String, String>,
    description: Option<String>,
}

impl Project {
    /// Constructs a new `Project`.
    ///
    /// This function is generic over its iterable inputs. It checks uniqueness of all IDs and
    /// performs graph integrity validation to ensure no dangling references exist.
    pub fn new(
        ind: impl Into<String>,
        samples: impl IntoIterator<Item = Arc<Sample>>,
        libraries: impl IntoIterator<Item = Arc<Library>>,
        experiments: impl IntoIterator<Item = Arc<Experiment>>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        let ind = parse::ind(ind)?;
        let meta = parse::meta(meta)?;
        let description = parse::non_empty_string("Project::description", description)?;

        let samples: Vec<Arc<Sample>> = samples.into_iter().collect();
        let libraries: Vec<Arc<Library>> = libraries.into_iter().collect();
        let experiments: Vec<Arc<Experiment>> = experiments.into_iter().collect();

        // Validate that we have at least one of each top-level entity
        validate::non_empty_collection("Project::samples", &samples)?;
        validate::non_empty_collection("Project::libraries", &libraries)?;
        validate::non_empty_collection("Project::experiments", &experiments)?;

        // Validate ID uniqueness
        let buffer = [ind.as_str()];
        let allids = buffer
            .iter()
            .map(|x| *x)
            .chain(samples.iter().map(|x| x.ind()))
            .chain(libraries.iter().map(|x| x.ind()))
            .chain(experiments.iter().map(|x| x.ind()))
            .chain(
                experiments
                    .iter()
                    .map(|x| x.runs())
                    .flatten()
                    .map(|x| x.ind()),
            );
        validate::unique_ids("Project", allids)?;

        // Validate graph integrity (no dangling references to samples/libraries)
        validate::validate_graph_integrity(&samples, &libraries, &experiments)?;

        Ok(Self {
            ind,
            samples,
            libraries,
            experiments,
            meta,
            description,
        })
    }

    /// Returns the project's unique identifier.
    pub fn ind(&self) -> &str {
        &self.ind
    }

    /// Returns a slice of the `Sample`s in this project.
    pub fn samples(&self) -> &[Arc<Sample>] {
        &self.samples
    }

    /// Returns a slice of the `Library`s in this project.
    pub fn libraries(&self) -> &[Arc<Library>] {
        &self.libraries
    }

    /// Returns a slice of the `Experiment`s in this project.
    pub fn experiments(&self) -> &[Arc<Experiment>] {
        &self.experiments
    }

    /// Returns a reference to the metadata map.
    pub fn meta(&self) -> &BTreeMap<String, String> {
        &self.meta
    }

    /// Returns the description as an `Option<&str>`.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

#[cfg(test)]
impl Project {
    pub fn dummy() -> Self {
        let experiments = Experiment::dummies()
            .into_iter()
            .map(Arc::new)
            .collect_vec();
        let libraries = experiments
            .iter()
            .map(|x| x.library().clone())
            .unique_by(|x| x.ind().to_string())
            .collect_vec();
        let samples = libraries
            .iter()
            .map(|x| x.sample().clone())
            .unique_by(|x| x.ind().to_string())
            .collect_vec();

        Self::new(
            "MockProject1",
            samples,
            libraries,
            experiments,
            [("ProjectMeta1", "MetaValue")],
            Some("This is a mock project"),
        )
        .unwrap()
    }
}

impl Debug for Project {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Project")
            .field("ind", &self.ind)
            .field(
                "samples",
                &self.samples.iter().map(|s| s.ind()).collect_vec(),
            )
            .field(
                "libraries",
                &self.libraries.iter().map(|l| l.ind()).collect_vec(),
            )
            .field(
                "experiments",
                &self.experiments.iter().map(|e| e.ind()).collect_vec(),
            )
            .finish()
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Project ID: {}", self.ind)?;
        writeln!(
            f,
            "Samples: {}",
            self.samples.iter().map(|s| s.ind()).join(", ")
        )?;
        writeln!(
            f,
            "Libraries: {}",
            self.libraries.iter().map(|l| l.ind()).join(", ")
        )?;
        writeln!(
            f,
            "Experiments: {}",
            self.experiments.iter().map(|e| e.ind()).join(", ")
        )?;
        if let Some(description) = &self.description {
            writeln!(f, "Description: {}", description)?;
        }
        if !self.meta.is_empty() {
            writeln!(f, "Meta:")?;
            for (k, v) in &self.meta {
                writeln!(f, "    {}: {}", k, v)?;
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct DeserializedProject {
    ind: String,
    samples: Vec<Sample>,
    libraries: Vec<DeserializedLibrary>,
    experiments: Vec<DeserializedExperiment>,
    meta: BTreeMap<String, String>,
    description: Option<String>,
}

impl<'de> Deserialize<'de> for Project {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let flat = DeserializedProject::deserialize(deserializer)?;

        // Recover all components in order, since later components depend on earlier ones.
        let mut ind2samples = HashMap::with_capacity(flat.samples.len());
        let samples = flat
            .samples
            .into_iter()
            .map(|x| {
                let arc = Arc::new(x);
                ind2samples.insert(arc.ind().to_owned(), arc.clone());
                arc
            })
            .collect_vec();

        let mut ind2libraries = HashMap::with_capacity(flat.libraries.len());
        let libraries = flat
            .libraries
            .into_iter()
            .map(|x| {
                x.finalize(&ind2samples)
                    .map(|x| {
                        let x = Arc::new(x);
                        ind2libraries.insert(x.ind().to_owned(), x.clone());
                        x
                    })
                    .map_err(serde::de::Error::custom)
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let experiments = flat
            .experiments
            .into_iter()
            .map(|x| {
                x.finalize(&ind2libraries)
                    .map(|x| Arc::new(x))
                    .map_err(serde::de::Error::custom)
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Project::new(
            flat.ind,
            samples,
            libraries,
            experiments,
            flat.meta,
            flat.description,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_display() {
        let mock = Project::dummy();
        let expected = "Project ID: MockProject1
Samples: Sample1, M2
Libraries: library, Lib-1
Experiments: Exp1, Exp2
Description: This is a mock project
Meta:
    ProjectMeta1: MetaValue
";
        assert_eq!(mock.to_string(), expected);
    }
}
