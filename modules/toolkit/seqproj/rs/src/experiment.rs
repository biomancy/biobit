use super::library::Library;
use super::run::Run;
use super::sample::Sample;
use super::{parse, validate};
use biobit_core_rs::ngs;
use biobit_core_rs::ngs::Layout;
use eyre::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug, Display, Formatter};
use std::sync::Arc;

/// Represents a sequencing experiment: the application of a
/// specific sequencing strategy to a `Library`.
///
/// An `Experiment` is a core entity in a `Project`. It serves as the
/// "bridge" linking a single `Library` (the "what was prepped")
/// to one or more `Run`s (the "how it was sequenced").
///
/// It holds the conceptual `ngs::Layout` (the "what") and enforces that
/// all its `Run`s have a compatible `Storage` (the "how"). For example,
/// an `Experiment` with a `ngs::Layout::Paired` *must* contain `Run`s
/// with `Storage::PairedFastq`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Experiment {
    ind: String,
    #[serde(serialize_with = "crate::serialization::library_ind")]
    library: Arc<Library>,
    layout: Layout,
    runs: Vec<Run>,
    meta: BTreeMap<String, String>,
    description: Option<String>,
}

impl Experiment {
    /// Constructs a new `Experiment`.
    ///
    /// This function is generic over its iterable inputs for `meta`.
    /// It performs critical validation to ensure the integrity of the
    /// project graph:
    ///
    /// *  Validates `ind`, `meta`, and `description` to ensure they are not empty strings.
    /// *  Ensures at least one `Run` is provided.
    /// *  Ensures all `Run` IDs within this experiment are unique.
    /// *  **Layout/Storage Bridge:** Validates that the `ngs::Layout` (the "what")
    ///     is compatible with the `Storage` (the "how") of every `Run`.
    pub fn new(
        ind: impl Into<String>,
        library: Arc<Library>,
        layout: impl Into<ngs::Layout>,
        runs: impl IntoIterator<Item = Run>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        let ind = parse::ind(ind)?;
        let meta = parse::meta(meta)?;
        let description = parse::non_empty_string("Experiment::description", description)?;
        let runs: Vec<Run> = runs.into_iter().collect();

        validate::non_empty_collection("Experiment::runs", &runs)?;
        validate::unique_ids("Experiment::runs::ind", runs.iter().map(|r| r.ind()))?;

        let layout = layout.into();
        validate::layout_storage_compatibility(&ind, &layout, &runs)?;

        Ok(Self {
            ind,
            library,
            layout,
            runs,
            meta,
            description,
        })
    }

    /// Returns the experiment's unique identifier.
    pub fn ind(&self) -> &str {
        &self.ind
    }

    /// Returns a shared reference to the parent `Library`.
    pub fn library(&self) -> &Arc<Library> {
        &self.library
    }

    /// Returns a convenience reference to the parent `Sample`
    /// (via `self.library.sample()`).
    pub fn sample(&self) -> &Arc<Sample> {
        self.library.sample()
    }

    /// Returns the conceptual `ngs::Layout` of this experiment.
    ///
    /// Each layout is associated with strandedness information, which
    /// might differ from the strandedness of the parent `Library` but is
    /// the source of truth for this `Experiment`.
    pub fn layout(&self) -> ngs::Layout {
        self.layout
    }

    /// Returns a slice of the `Run`s owned by this experiment.
    pub fn runs(&self) -> &[Run] {
        &self.runs
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
impl Experiment {
    pub fn dummy() -> Self {
        Self::new(
            "Exp1",
            Arc::new(Library::dummy()),
            ngs::Layout::Single {
                strandedness: ngs::Strandedness::Unstranded,
            },
            vec![Run::dummy()],
            [("Project", "Genome Study")],
            Some("A mock experiment"),
        )
        .unwrap()
    }

    pub fn dummies() -> Vec<Self> {
        let libraries = Library::dummies();
        vec![
            Self::dummy(),
            Self::new(
                "Exp2",
                Arc::new(libraries[1].clone()),
                ngs::Layout::Paired {
                    strandedness: ngs::Strandedness::Forward,
                    orientation: ngs::MatesOrientation::Inward,
                },
                Run::dummies(),
                Vec::<(String, String)>::new(),
                Some("This is another mock experiment"),
            )
            .unwrap(),
        ]
    }
}

impl Debug for Experiment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Experiment(ind: {}, library: {}, runs: [{}])",
            self.ind,
            self.library.ind(),
            self.runs.iter().map(|r| r.ind()).join(", ")
        )
    }
}

impl Display for Experiment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Experiment ID: {}", self.ind)?;
        writeln!(f, "Library: {}", self.library.ind())?;
        writeln!(f, "Layout: {:?}", self.layout)?;
        writeln!(f, "Runs:")?;
        for run in &self.runs {
            writeln!(f, "    {}", run.ind())?;
        }
        if !self.meta.is_empty() {
            writeln!(f, "Meta:")?;
            for (k, v) in &self.meta {
                writeln!(f, "    {}: {}", k, v)?;
            }
        }
        if let Some(description) = &self.description {
            writeln!(f, "Description: {}", description)?;
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct DeserializedExperiment {
    ind: String,
    library: String,
    layout: Layout,
    runs: Vec<Run>,
    meta: BTreeMap<String, String>,
    description: Option<String>,
}

impl DeserializedExperiment {
    pub fn finalize(self, libraries: &HashMap<String, Arc<Library>>) -> Result<Experiment> {
        let library = libraries
            .get(&self.library)
            .ok_or_else(|| {
                eyre::eyre!(
                    "Library ID '{}' not found during Experiment deserialization",
                    self.library
                )
            })?
            .clone();

        Experiment::new(
            self.ind,
            library,
            self.layout,
            self.runs,
            self.meta,
            self.description,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use biobit_core_rs::ngs::Strandedness;
    #[test]
    fn test_experiment_new() {
        let library = Arc::new(Library::dummy());
        let run = Run::dummy();
        let layout = ngs::Layout::Single {
            strandedness: Strandedness::Forward,
        };
        let exp = Experiment::new(
            "Experiment",
            library.clone(),
            layout,
            vec![run.clone()],
            [("project", "Cancer")],
            None::<String>,
        )
        .unwrap();

        assert_eq!(exp.ind(), "Experiment");
        assert_eq!(exp.library(), &library);
        assert_eq!(exp.sample(), library.sample());
        assert_eq!(exp.layout(), layout);
        assert_eq!(exp.runs(), vec![run]);
        assert_eq!(
            exp.meta(),
            &BTreeMap::from([("project".to_string(), "Cancer".to_string())])
        );
        assert_eq!(exp.description(), None);
    }

    #[test]
    fn test_experiment_new_fail() {
        let template = Experiment::dummy();
        assert!(
            Experiment::new(
                "",
                template.library().clone(),
                template.layout(),
                template.runs().to_vec(),
                template.meta(),
                template.description(),
            )
            .is_err(),
            "Empty ind"
        );

        assert!(
            Experiment::new(
                template.ind(),
                template.library().clone(),
                template.layout(),
                Vec::<Run>::new(),
                template.meta(),
                template.description(),
            )
            .is_err(),
            "Empty runs"
        );

        assert!(
            Experiment::new(
                template.ind(),
                template.library().clone(),
                template.layout(),
                [Run::dummy(), Run::dummy()],
                template.meta(),
                template.description(),
            )
            .is_err(),
            "Duplicate runs"
        );
    }

    #[test]
    fn test_experiment_display() {
        let experiment = Experiment::dummies()
            .into_iter()
            .map(|x| x.clone())
            .join("");
        let expected = [
            "Experiment ID: Exp1
Library: library
Layout: Single { strandedness: Unstranded }
Runs:
    MockRun
Meta:
    Project: Genome Study
Description: A mock experiment",
            "Experiment ID: Exp2
Library: Lib-1
Layout: Paired { strandedness: Forward, orientation: Inward }
Runs:
    MockRun1
    MockRun2
Description: This is another mock experiment
",
        ]
        .join("\n");
        assert_eq!(experiment, expected);
    }
}
