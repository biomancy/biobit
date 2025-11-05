use super::parse;
use super::storage::Storage;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::num::NonZeroU64;

/// Represents a single sequencing run, a component of an `Experiment`.
///
/// A `Run` is a concrete execution of a sequencing protocol,
/// which produces one or more data files. It is owned by an `Experiment`.
///
/// Each `Run` has a unique identifier (`ind`) within the project,
/// a `Storage` descriptor detailing its storage method, and optional
/// metadata (`meta`) and a `description`. It may also include optional
/// fields for the sequencing machine used, total generated reads, and sequenced bases.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Run {
    ind: String,
    storage: Storage,
    machine: Option<String>,
    reads: Option<NonZeroU64>,
    bases: Option<NonZeroU64>,
    meta: BTreeMap<String, String>,
    description: Option<String>,
}

impl Run {
    /// Constructs a new `Run`.
    ///
    /// This function is generic over its iterable inputs for `meta`.
    /// It validates all inputs to ensure that:
    /// 1. `ind` cannot be empty.
    /// 2. No key or value in `meta` can be an empty string.
    /// 3. `description`, if `Some`, cannot be an empty string.
    pub fn new(
        ind: impl Into<String>,
        storage: impl Into<Storage>,
        machine: Option<impl Into<String>>,
        reads: Option<impl Into<u64>>,
        bases: Option<impl Into<u64>>,
        meta: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        Ok(Self {
            ind: parse::ind(ind)?,
            storage: storage.into(),
            machine: parse::non_empty_string("Run::machine", machine)?,
            reads: parse::non_zero_u64("Run::reads", reads)?,
            bases: parse::non_zero_u64("Run::bases", bases)?,
            meta: parse::meta(meta)?,
            description: parse::non_empty_string("Run::description", description)?,
        })
    }

    /// Returns the run's unique identifier.
    ///
    /// Uniqueness is guaranteed within a `Project`.
    pub fn ind(&self) -> &str {
        &self.ind
    }

    /// Returns a reference to the `RunStorage`.
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Returns the sequencing machine as an `Option<&str>`.
    pub fn machine(&self) -> Option<&str> {
        self.machine.as_deref()
    }

    /// Returns the total number of reads as an `Option<NonZeroU64>`.
    pub fn reads(&self) -> Option<NonZeroU64> {
        self.reads
    }

    /// Returns the total number of bases as an `Option<NonZeroU64>`.
    pub fn bases(&self) -> Option<NonZeroU64> {
        self.bases
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
impl Run {
    pub fn dummy() -> Self {
        Self::new(
            "MockRun",
            Storage::SingleFastq {
                file: "mock_run.fq".into(),
            },
            Some("MockMachine"),
            Some(1000u64),
            Some(150000u64),
            Vec::<(String, String)>::new(),
            Some("This is a mock run for testing."),
        )
        .unwrap()
    }

    pub fn dummies() -> Vec<Self> {
        vec![
            Self::new(
                "MockRun1",
                Storage::PairedFastq {
                    file1: "mock_run-1.fq".into(),
                    file2: "mock_run-2.fq".into(),
                },
                Some("MockMachine"),
                Some(1000u64),
                Some(150000u64),
                Vec::<(String, String)>::new(),
                Some("This is a mock run for testing."),
            )
            .unwrap(),
            Self::new(
                "MockRun2",
                Storage::PairedFastq {
                    file1: "mock_run2_R1.fq".into(),
                    file2: "mock_run2_R2.fq".into(),
                },
                None::<String>,
                None::<u64>,
                Some(3010u64),
                [("key1", "value1"), ("key2", "value2")],
                None::<String>,
            )
            .unwrap(),
        ]
    }
}

impl Debug for Run {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Run({})", self.ind)
    }
}

impl Display for Run {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Run ID: {}", self.ind)?;
        writeln!(f, "Storage: {}", self.storage)?;
        if let Some(machine) = &self.machine {
            writeln!(f, "Machine: {}", machine)?;
        }
        if let Some(reads) = &self.reads {
            writeln!(f, "Reads: {}", reads)?;
        }
        if let Some(bases) = &self.bases {
            writeln!(f, "Bases: {}", bases)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::Sample;

    #[test]
    fn test_new() {
        let ind = "Run1";
        let storage = Storage::PairedFastq {
            file1: "run1_R1.fq.gz".into(),
            file2: "run1_R2.fq.gz".into(),
        };
        let machine = Some("Illumina NovaSeq 6000");
        let reads = Some(1_000_000u64);
        let bases = Some(3_000_000_000u64);
        let meta = [("flowcell", "FC-123"), ("lane", "L1")];
        let description = Some("First lane");

        let run = Run::new(
            ind,
            storage.clone(),
            machine,
            reads,
            bases,
            meta,
            description,
        )
        .unwrap();

        assert_eq!(run.ind(), ind);
        assert_eq!(run.storage(), &storage);
        assert_eq!(run.machine(), machine);
        assert_eq!(run.reads().map(|x| x.get()), reads);
        assert_eq!(run.bases().map(|x| x.get()), bases);
        assert_eq!(
            run.meta(),
            &meta
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .into_iter()
                .collect()
        );
        assert_eq!(run.description(), description);

        // Ensure tha other mock samples can be created
        Sample::dummies();
    }

    #[test]
    fn test_new_fails() {
        let run = Run::dummy();

        assert!(
            Run::new(
                "",
                run.storage().clone(),
                run.machine(),
                run.reads(),
                run.bases(),
                run.meta(),
                run.description(),
            )
            .is_err(),
            "Empty ind"
        );

        assert!(
            Run::new(
                run.ind(),
                run.storage().clone(),
                run.machine(),
                run.reads(),
                run.bases(),
                [("", "value")],
                run.description(),
            )
            .is_err(),
            "Empty meta key"
        );

        assert!(
            Run::new(
                run.ind(),
                run.storage().clone(),
                run.machine(),
                run.reads(),
                run.bases(),
                [("key", "")],
                run.description(),
            )
            .is_err(),
            "Empty meta value"
        );

        assert!(
            Run::new(
                run.ind(),
                run.storage().clone(),
                run.machine(),
                run.reads(),
                run.bases(),
                run.meta(),
                Some("")
            )
            .is_err(),
            "Empty description"
        );
    }

    #[test]
    fn test_run_display() {
        let mock = Run::dummy();
        let expected = "Run ID: MockRun
Storage: SingleFastq(mock_run.fq)
Machine: MockMachine
Reads: 1000
Bases: 150000
Description: This is a mock run for testing.
";
        assert_eq!(mock.to_string(), expected);
    }
}
