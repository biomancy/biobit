use super::validate;
#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::{Dissolve, Getters};
use eyre::Result;
use std::error::Error;

pub trait RecordOp {
    fn id(&self) -> &str;
    fn seq(&self) -> &[u8];
}

pub trait RecordMutOp {
    fn set_id(&mut self, id: String) -> Result<&mut Self>;
    fn set_seq(&mut self, seq: Vec<u8>) -> Result<&mut Self>;
}

/// A single FASTA record with the following guarantees:
/// - The ID is non-empty and is represented by an arbitrary UTF-8 string.
/// - The ID can't contain any newline characters (CR or LF).
/// - The ID should end with LF or CR-LF.
/// - The sequence must be non-empty and contain only ASCII alphabetic characters.
///
/// There are no guarantees on the biological meaningfulness of the stored sequence. For example,
/// protein sequences could be mixed with DNA sequences and that would be a valid record.
#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Dissolve, Getters)]
pub struct Record {
    id: String,
    seq: Vec<u8>,
}

impl Record {
    /// Creates a new FASTA record with the given ID and sequence.
    pub fn new(id: String, seq: Vec<u8>) -> Result<Self> {
        validate::id(&id)?;
        validate::seq(&seq)?;
        Ok(Self { id, seq })
    }

    /// # Safety
    /// The caller must ensure that the ID and sequence are valid.
    pub unsafe fn new_unchecked(id: String, seq: Vec<u8>) -> Self {
        Self { id, seq }
    }

    /// # Safety
    /// The caller must ensure that all fields remains valid after modifications.
    pub unsafe fn fields(&mut self) -> (&mut String, &mut Vec<u8>) {
        (&mut self.id, &mut self.seq)
    }
}

impl RecordOp for Record {
    fn id(&self) -> &str {
        &self.id
    }

    fn seq(&self) -> &[u8] {
        &self.seq
    }
}

impl RecordMutOp for Record {
    fn set_id(&mut self, id: String) -> Result<&mut Self> {
        validate::id(&id)?;
        self.id = id;
        Ok(self)
    }

    fn set_seq(&mut self, seq: Vec<u8>) -> Result<&mut Self> {
        validate::seq(&seq)?;
        self.seq = seq;
        Ok(self)
    }
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
