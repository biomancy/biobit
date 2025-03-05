#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::{Dissolve, Getters};
use derive_more::Into;
use eyre::{ensure, Result};
use std::error::Error;

/// A single FASTA record with the following guarantees:
/// - The ID is non-empty and is represented by an arbitrary UTF-8 string.
/// - The ID can't contain any newline characters (CR or LF).
/// - The ID should end with LF or CR-LF.
/// - The sequence must be non-empty and contain only ASCII alphabetic characters.
///
/// There are no guarantees on the biological meaningfulness of the stored sequence. For example,
/// protein sequences could be mixed with DNA sequences and that would be a valid record.
#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Dissolve, Getters, Into)]
pub struct Record {
    id: String,
    seq: Vec<u8>,
}

impl Default for Record {
    fn default() -> Self {
        Self {
            id: "Default ID".to_string(),
            seq: b"ACGT".to_vec(),
        }
    }
}

impl<ID, SEQ> TryFrom<(ID, SEQ)> for Record
where
    ID: TryInto<String, Error: Error + Send + Sync + 'static>,
    SEQ: TryInto<Vec<u8>, Error: Error + Send + Sync + 'static>,
{
    type Error = eyre::Report;

    fn try_from(value: (ID, SEQ)) -> Result<Self> {
        Self::new(value.0.try_into()?, value.1.try_into()?)
    }
}

impl Record {
    /// Creates a new FASTA record with the given ID and sequence.
    pub fn new(id: String, seq: Vec<u8>) -> Result<Self> {
        Self::validate(&id, &seq)?;
        Ok(Self { id, seq })
    }

    pub fn validate_id(id: &str) -> Result<()> {
        ensure!(!id.is_empty(), "FASTA ID cannot be empty");
        ensure!(
            !id.contains(&['\n', '\r'] as &[char]),
            "Newline characters are not allowed in the FASTA ID: {id}"
        );
        Ok(())
    }

    pub fn validate_seq(seq: &[u8]) -> Result<()> {
        ensure!(!seq.is_empty(), "FASTA sequence cannot be empty");
        for (i, &x) in seq.iter().enumerate() {
            ensure!(
                x.is_ascii_alphabetic(),
                "Non-alphabetic character at index {i} = {x:?}"
            );
        }
        Ok(())
    }

    pub fn validate(id: &str, seq: &[u8]) -> Result<()> {
        Self::validate_id(id)?;
        Self::validate_seq(seq)
    }

    /// # Safety
    /// The caller must ensure that the ID and sequence are valid.
    pub unsafe fn new_unchecked(id: String, seq: Vec<u8>) -> Self {
        Self { id, seq }
    }

    /// # Safety
    /// The caller must ensure that all fields remains valid after modification.
    pub unsafe fn raw(&mut self) -> (&mut String, &mut Vec<u8>) {
        (&mut self.id, &mut self.seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() -> Result<()> {
        for (id, seq) in [
            ("Normal-header", "ACGTACGT"),
            ("id ", "a"),
            (" id ", "O"),
            ("internal 💉 spaces and emoji here", "ACGT"),
        ] {
            let record: Record = (id, seq).try_into()?;
            assert_eq!(record.id(), id);
            assert_eq!(record.seq(), seq.as_bytes());
        }

        Ok(())
    }

    #[test]
    fn test_invalid_records() {
        for (id, seq) in [
            // Invalid ID
            ("", "ACGT"),
            ("id\n", "ACGT"),
            ("id\r", "ACGT"),
            ("id\r\n", "ACGT"),
            // Invalid sequence
            ("id", ""),
            ("id", "ACGT1"),
            ("id", " ACGT"),
            ("id", "ACG T"),
            ("id", "ACGT "),
        ] {
            let record: Result<Record> = (id, seq).try_into();
            assert!(record.is_err(), "Record: {:?}", record);
        }
    }
}
