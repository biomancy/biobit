use super::sample::Sample;
use biobit_core_rs::ngs::Strandedness;

use super::parse;
use eyre::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

/// Represents a sequencing library, derived from a biological `Sample`.
///
/// A `Library` is a core entity in a `Project`. It defines the
/// specific molecular preparation (e.g., "poly-A selected, stranded RNA-seq")
/// that was performed on a `Sample`.
///
/// Each `Library` has a unique identifier (`ind`), a direct, shared
/// reference to its parent `Sample`, and defines its molecular
/// `source` (e.g., "RNA"), `selection` (e.g., "poly-A"),
/// `strandedness` (i.e., how the library molecule corresponds to the original NA molecule),
/// and optional metadata (`meta`) and a `description`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Library {
    ind: String,
    #[serde(serialize_with = "crate::serialization::sample_ind")]
    sample: Arc<Sample>,
    source: BTreeSet<String>,
    selection: BTreeSet<String>,
    strandedness: Option<Strandedness>,
    meta: BTreeMap<String, String>,
    description: Option<String>,
}

impl Library {
    /// Constructs a new `Library`.
    ///
    /// This function is generic over its iterable inputs to provide an ergonomic API.
    /// It validates all inputs to ensure that:
    /// 1. `ind` cannot be empty.
    /// 2. `source` cannot be empty, nor can any of its elements.
    /// 3. `selection` cannot be empty, nor can any of its elements.
    /// 4. No key or value in `meta` can be an empty string.
    /// 5. `description`, if `Some`, cannot be an empty string.
    ///
    /// The `sample` is passed as an `Arc<Sample>` to enforce the
    /// shared ownership model.
    pub fn new(
        ind: impl Into<String>,
        sample: Arc<Sample>,
        source: impl IntoIterator<Item = impl Into<String>>,
        selection: impl IntoIterator<Item = impl Into<String>>,
        strandedness: Option<Strandedness>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            ind: parse::ind(ind)?,
            sample,
            source: parse::set_of_non_empty_strings("Library::source", source)?,
            selection: parse::set_of_non_empty_strings("Library::selection", selection)?,
            strandedness,
            meta: parse::meta(meta)?,
            description: parse::non_empty_string("Library::description", description)?,
        })
    }

    /// Returns the library's unique identifier.
    ///
    /// Uniqueness is guaranteed within a `Project`.
    pub fn ind(&self) -> &str {
        &self.ind
    }

    /// Returns a shared reference to the parent `Sample`.
    pub fn sample(&self) -> &Arc<Sample> {
        &self.sample
    }

    /// Returns a reference to the set of source molecules.
    pub fn source(&self) -> &BTreeSet<String> {
        &self.source
    }

    /// Returns a reference to the set of selection methods.
    pub fn selection(&self) -> &BTreeSet<String> {
        &self.selection
    }

    /// Returns the `Strandedness` (if known).
    ///
    /// Note that strandedness af a sequenced library may change depending on
    /// the sequencing protocol used. The final strandedness is recorded in the
    /// `Experiment::layout` derived from this `Library`.
    pub fn strandedness(&self) -> Option<Strandedness> {
        self.strandedness
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

#[derive(Deserialize)]
pub struct DeserializedLibrary {
    ind: String,
    sample: String,
    source: BTreeSet<String>,
    selection: BTreeSet<String>,
    strandedness: Option<Strandedness>,
    meta: BTreeMap<String, String>,
    description: Option<String>,
}

impl DeserializedLibrary {
    pub fn finalize(self, samples: &HashMap<String, Arc<Sample>>) -> Result<Library> {
        let sample = samples
            .get(&self.sample)
            .ok_or_else(|| {
                eyre::eyre!(
                    "Sample ID '{}' not found during Library deserialization",
                    self.sample
                )
            })?
            .clone();
        Library::new(
            self.ind,
            sample,
            self.source,
            self.selection,
            self.strandedness,
            self.meta,
            self.description,
        )
    }
}

#[cfg(test)]
impl Library {
    pub fn dummy() -> Self {
        Self::new(
            "library",
            Arc::new(Sample::dummy()),
            ["RNA"],
            ["poly-A"],
            Some(Strandedness::Forward),
            [("kit", "TruSeq")],
            Some("A library"),
        )
        .unwrap()
    }

    pub fn dummies() -> Vec<Self> {
        let samples = Sample::dummies().into_iter().map(Arc::new).collect_vec();
        vec![
            Self::dummy(),
            Self::new(
                "Lib-1",
                samples[1].clone(),
                ["RNA"],
                ["poly-A"],
                Some(Strandedness::Forward),
                [("kit", "TruSeq")],
                Some("A library2"),
            )
            .unwrap(),
            Self::new(
                "Lib-2",
                samples[2].clone(),
                ["DNA"],
                ["none"],
                None,
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .unwrap(),
        ]
    }
}

impl Debug for Library {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Library(ind: {}, sample: {})",
            self.ind,
            self.sample.ind()
        )
    }
}

impl Display for Library {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Library ID: {}", self.ind)?;
        writeln!(f, "Sample: {}", self.sample.ind())?;
        writeln!(f, "Source: {}", self.source.iter().join(", "))?;
        writeln!(f, "Selection: {}", self.selection.iter().join(", "))?;
        match self.strandedness {
            Some(s) => writeln!(f, "Strandedness: {s}"),
            None => writeln!(f, "Strandedness: Unknown"),
        }?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_new() {
        let sample = Arc::new(Sample::dummy());
        let source = ["transcriptome", "mRNA"];
        let selection = ["poly-A", "size-selection"];
        let meta = [("kit", "TruSeq"), ("operator", "Jane")];
        let lib = Library::new(
            "L1",
            sample.clone(),
            source,
            selection,
            Some(Strandedness::Forward),
            meta,
            Some("Test library"),
        )
        .unwrap();

        assert_eq!(lib.ind(), "L1");
        assert_eq!(lib.sample(), &sample);
        assert_eq!(Arc::strong_count(lib.sample()), 2);
        assert_eq!(
            lib.source(),
            &source.map(ToString::to_string).into_iter().collect()
        );
        assert_eq!(
            lib.selection(),
            &selection.map(ToString::to_string).into_iter().collect()
        );
        assert_eq!(lib.strandedness(), Some(Strandedness::Forward));
        assert_eq!(
            lib.meta(),
            &meta
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .into_iter()
                .collect()
        );
        assert_eq!(lib.description(), Some("Test library"));
    }

    #[test]
    fn test_library_new_fails() {
        let template = Library::dummy();

        assert!(
            Library::new(
                "",
                template.sample().clone(),
                template.source(),
                template.selection(),
                template.strandedness(),
                template.meta(),
                template.description()
            )
            .is_err(),
            "Empty ind"
        );

        assert!(
            Library::new(
                template.ind(),
                template.sample().clone(),
                Vec::<String>::new(),
                template.selection(),
                template.strandedness(),
                template.meta(),
                template.description()
            )
            .is_err(),
            "Empty source"
        );

        assert!(
            Library::new(
                template.ind(),
                template.sample().clone(),
                template.source(),
                Vec::<String>::new(),
                template.strandedness(),
                template.meta(),
                template.description()
            )
            .is_err(),
            "Empty selection"
        );

        assert!(
            Library::new(
                template.ind(),
                template.sample().clone(),
                template.source(),
                template.selection(),
                template.strandedness(),
                [("", "value")],
                template.description()
            )
            .is_err(),
            "Empty meta key"
        );

        assert!(
            Library::new(
                template.ind(),
                template.sample().clone(),
                template.source(),
                template.selection(),
                template.strandedness(),
                [("key", "")],
                template.description()
            )
            .is_err(),
            "Empty meta value"
        );

        assert!(
            Library::new(
                template.ind(),
                template.sample().clone(),
                template.source(),
                template.selection(),
                template.strandedness(),
                template.meta(),
                Some(""),
            )
            .is_err(),
            "Empty description",
        );
    }

    #[test]
    fn test_library_display() {
        let dummies = Library::dummies();

        let expected = vec![
            "Library ID: library
Sample: Sample1
Source: RNA
Selection: poly-A
Strandedness: Forward
Meta:
    kit: TruSeq
Description: A library
",
            "Library ID: Lib-1
Sample: M2
Source: RNA
Selection: poly-A
Strandedness: Forward
Meta:
    kit: TruSeq
Description: A library2
",
            "Library ID: Lib-2
Sample: M3
Source: DNA
Selection: none
Strandedness: Unknown
",
        ];

        assert_eq!(
            expected.join("\n"),
            dummies.iter().map(ToString::to_string).join("\n")
        );
    }
}
