use crate::parse;
use eyre::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

/// Represents a biological sample, the source of a sequencing library.
///
/// A `Sample` is a core entity in a `Project`. It defines the physical
/// biomaterial (e.g., "Tumor tissue from Patient X") from which one
/// or more `Library` objects are derived.
///
/// Each `Sample` has a unique identifier (`ind`) within the project,
/// a set of associated organisms, and optional metadata (`meta`)
/// and a `description`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Sample {
    ind: String,
    organisms: BTreeSet<String>,
    meta: BTreeMap<String, String>,
    description: Option<String>,
}

impl Sample {
    /// Constructs a new `Sample`.
    ///
    /// This function is generic over its inputs to provide an ergonomic API.
    /// It validates all inputs to ensure that:
    /// 1. `ind` cannot be empty.
    /// 2. `organisms` cannot be empty, nor can any of its elements.
    /// 3. No key or value in `meta` can be an empty string.
    /// 4. `description`, if `Some`, cannot be an empty string.
    pub fn new(
        ind: impl Into<String>,
        organisms: impl IntoIterator<Item = impl Into<String>>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            ind: parse::ind(ind)?,
            organisms: parse::set_of_non_empty_strings("Sample::organisms", organisms)?,
            meta: parse::meta(meta)?,
            description: parse::non_empty_string("Sample::description", description)?,
        })
    }

    /// Returns the sample's unique identifier.
    ///
    /// Each Sample must have an identifier that is unique within a given Project.
    pub fn ind(&self) -> &str {
        &self.ind
    }

    /// Returns a reference to the set of organisms.
    pub fn organisms(&self) -> &BTreeSet<String> {
        &self.organisms
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
impl Sample {
    pub fn dummy() -> Self {
        Sample::new(
            "Sample1",
            ["Homo sapiens", "E. coli"],
            [("tissue", "liver"), ("treatment", "none")],
            Some("Mock sample"),
        )
        .unwrap()
    }

    pub fn dummies() -> Vec<Self> {
        vec![
            Self::dummy(),
            Self::new(
                "M2",
                ["E. coli"],
                Vec::<(String, String)>::new(),
                None::<String>,
            )
            .unwrap(),
            Self::new(
                "M3",
                ["Homo sapiens"],
                [("tissue", "heart")],
                Some("Mock sample M3"),
            )
            .unwrap(),
        ]
    }
}

impl Debug for Sample {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Sample({})", self.ind)
    }
}

impl Display for Sample {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Sample ID: {}", self.ind)?;
        writeln!(f, "Organisms: {}", self.organisms.iter().join(", "))?;
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
    fn test_sample_new_ok() {
        let sample = Sample::dummy();
        assert_eq!(sample.ind(), "Sample1");
        assert_eq!(
            sample.organisms().iter().collect_vec(),
            ["E. coli", "Homo sapiens"]
        );
        assert_eq!(
            sample
                .meta()
                .iter()
                .map(|x| (x.0.as_str(), x.1.as_str()))
                .collect_vec(),
            [("tissue", "liver"), ("treatment", "none")]
        );
        assert_eq!(sample.description(), Some("Mock sample"));

        // Ensure tha other mock samples can be created
        Sample::dummies();
    }

    #[test]
    fn test_new_fails() {
        let ind = "IND";
        let organisms = ["Homo sapiens"];
        let meta = [("key", "value")];
        let description = Some("Description");

        let res = Sample::new("", organisms, meta, description);
        assert!(res.is_err());

        let res = Sample::new(ind, Vec::<String>::new(), meta, description);
        assert!(res.is_err());

        let res = Sample::new(ind, ["Homo sapiens", ""], meta, description);
        assert!(res.is_err());

        let err_meta = [("", "value")];
        let res = Sample::new(ind, organisms, err_meta, description);
        assert!(res.is_err());

        let err_meta = [("key", "")];
        let res = Sample::new(ind, organisms, err_meta, description);
        assert!(res.is_err());

        let res = Sample::new(ind, organisms, meta, Some(""));
        assert!(res.is_err());
    }

    #[test]
    fn test_sample_display() {
        let sample = Sample::new(
            "S_101",
            ["Mus musculus", "C. elegans"],
            [("tissue", "brain"), ("treatment", "drug_A")],
            Some("A test sample"),
        )
        .unwrap();

        // BTreeSet sorts the organisms alphabetically
        let expected = "Sample ID: S_101
Organisms: C. elegans, Mus musculus
Meta:
    tissue: brain
    treatment: drug_A
Description: A test sample
";
        assert_eq!(sample.to_string(), expected);

        let sample = Sample::new(
            "S_102",
            ["Homo sapiens"],
            Vec::<(String, String)>::new(),
            None::<String>,
        )
        .unwrap();

        let expected = "Sample ID: S_102
Organisms: Homo sapiens
";
        assert_eq!(sample.to_string(), expected);
    }
}
