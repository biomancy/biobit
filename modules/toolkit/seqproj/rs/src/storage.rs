use eyre::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Debug, Display};
use std::path::PathBuf;

/// Describes the physical storage layout for a single sequencing `Run`.
///
/// This enum captures the "how" a run's data is stored, separating the physical
/// file layout from the conceptual `ngs::Layout` (the "what").
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Storage {
    /// A single FASTQ file.
    ///
    /// Typically used for single-end experiments or formats
    /// where paired-end reads are interleaved.
    SingleFastq { file: PathBuf },

    /// A pair of FASTQ files.
    ///
    /// The standard for paired-end sequencing.
    PairedFastq { file1: PathBuf, file2: PathBuf },
}

impl Debug for Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl Display for Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Storage::SingleFastq { file } => {
                write!(f, "SingleFastq({})", file.display())
            }
            Storage::PairedFastq {
                file1: read1,
                file2: read2,
            } => {
                write!(f, "PairedFastq({}, {})", read1.display(), read2.display())
            }
        }
    }
}

impl Storage {
    /// Returns `Ok(true)` if all paths point at an existing entity.
    ///
    /// This function semantic follows thus of the [`std::path::Path::try_exists()`] function.
    pub fn try_exists(&self) -> Result<bool> {
        match self {
            Storage::SingleFastq { file } => Ok(file.try_exists()?),
            Storage::PairedFastq {
                file1: read1,
                file2: read2,
            } => Ok(read1.try_exists()? && read2.try_exists()?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_display() {
        // SingleFastq
        let single = Storage::SingleFastq {
            file: PathBuf::from("data/run.fastq.gz"),
        };
        assert_eq!(single.to_string(), "SingleFastq(data/run.fastq.gz)");

        // PairedFastq
        let paired = Storage::PairedFastq {
            file1: PathBuf::from("data/run_R1.fq"),
            file2: PathBuf::from("data/run_R2.fq"),
        };
        assert_eq!(
            paired.to_string(),
            "PairedFastq(data/run_R1.fq, data/run_R2.fq)"
        );
    }

    #[test]
    fn test_storage_equality_and_clone() {
        let single1 = Storage::SingleFastq {
            file: PathBuf::from("file.fq"),
        };
        let single2 = single1.clone();
        assert_eq!(single1, single2);

        let paired1 = Storage::PairedFastq {
            file1: PathBuf::from("r1.fq"),
            file2: PathBuf::from("r2.fq"),
        };
        let paired2 = Storage::PairedFastq {
            file1: PathBuf::from("r1.fq"),
            file2: PathBuf::from("r2.fq"),
        };
        assert_eq!(paired1, paired2);

        assert_ne!(single1, paired1);
    }
}
