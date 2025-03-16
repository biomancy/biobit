use super::{record::Record, validate};
use crate::compression;
use crate::compression::DecompressedStream;
use derive_getters::Dissolve;
use eyre::{ensure, Result};
use memchr;
use std::io::BufRead;
use std::path::Path;

/// A strict FASTA reader that can read a single record at a time. Ignores:
/// - Carriage return characters at the end of all lines (to support Windows line endings)
///
/// Returns an error if there are:
/// - Errors while reading from the underlying reader
/// - Extra characters before the first record, between records, or after the last record
/// - Non-alphabetic characters inside the sequence, including start/end of lines
/// - Empty ID or sequence fields in any record
pub trait ReaderMutOp {
    /// Parse the next FASTA record into the given [Record] buffer.
    /// Returns None if there are no more records to read.
    ///
    /// The read is successful only if the function returns `Ok(Some())`. Otherwise, the buffer is
    /// left in an unspecified state, but can be reused for the next read.
    fn read_record(&mut self, into: &mut Record) -> Result<Option<()>>;

    /// Read the remaining records in the file and place them into the given vector. Returns the
    /// number of records read.
    ///
    /// The function returns an error if there are any issues while reading the file.
    /// Records outside the ones read are left in an unspecified state but can be reused for the next read.
    fn read_to_end(&mut self, into: &mut Vec<Record>) -> Result<usize>;
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Dissolve)]
pub struct Reader<R> {
    reader: R,
}

impl<R: BufRead> Reader<R> {
    pub fn new(mut reader: R) -> Result<Self> {
        // Check that there are no extra characters before the first record
        let buffer = reader.fill_buf()?;
        ensure!(
            buffer.first().map(|x| *x == b'>').unwrap_or(true),
            "Expected '>' at the start of the FASTA file"
        );
        Ok(Self { reader })
    }

    /// Create a new FASTA reader from the given file path.
    /// The compression is automatically detected based on the file extension and the internal file signature.
    pub fn from_path(
        path: impl AsRef<Path>,
    ) -> Result<Box<dyn ReaderMutOp + Send + Sync + 'static>> {
        let boxed: Box<dyn ReaderMutOp + Send + Sync + 'static> =
            match compression::read_file(path)? {
                DecompressedStream::PlainText(file) => {
                    Box::new(Reader::new(std::io::BufReader::new(file))?)
                }
                DecompressedStream::Gzip(gzip) => {
                    Box::new(Reader::new(std::io::BufReader::new(gzip))?)
                }
            };

        Ok(boxed)
    }

    fn read_next(&mut self, id: &mut String, seq: &mut Vec<u8>) -> Result<Option<()>> {
        // Ensure that the next symbol is '>' and consume it
        let buffer = self.reader.fill_buf()?;
        if buffer.is_empty() {
            return Ok(None);
        }
        ensure!(
            buffer.first().map(|x| *x == b'>').unwrap_or(false),
            "Expected '>' at the start of the FASTA record"
        );
        self.reader.consume(1);

        // Read and validated the ID line
        id.clear();
        let read = self.reader.read_line(id)?;
        ensure!(read > 0, "Unexpected EOF after '>'");
        // Remove the trailing newline
        ensure!(
            id.ends_with('\n'),
            "FASTA ID line is not terminated with a newline : {id}"
        );
        id.pop();
        // Remove the trailing carriage return if present
        if id.ends_with('\r') {
            id.pop();
        }
        validate::id(id)?;

        // Read and validate the sequence lines
        seq.clear();
        loop {
            // Read the next line
            let buffer = self.reader.fill_buf()?;
            if buffer.is_empty() {
                break;
            }

            // If the buffer starts with '>', then it is the start of the next record
            if buffer[0] == b'>' {
                break;
            }

            // Find the end of the line if it exists
            let (line, consume) = match memchr::memchr(b'\n', buffer) {
                Some(pos) => {
                    let line = &buffer[..pos];
                    // Remove the trailing '\r' if needed
                    if line.last().map(|x| *x == b'\r').unwrap_or(false) {
                        (&line[..line.len() - 1], pos + 1)
                    } else {
                        (line, pos + 1)
                    }
                }
                None => (buffer, buffer.len()),
            };

            // Err if the line is empty
            ensure!(
                !line.is_empty(),
                "Empty line was found for the sequence with id {}",
                id
            );
            debug_assert!(!line.contains(&b'>'));

            // Append the line to the sequence
            seq.extend_from_slice(line);

            // Consume the processed bytes
            self.reader.consume(consume);
        }
        validate::seq(seq)?;

        Ok(Some(()))
    }
}

impl<R: BufRead> ReaderMutOp for Reader<R> {
    fn read_record(&mut self, into: &mut Record) -> Result<Option<()>> {
        // SAFETY: The ID and sequence are checked for validity before being written to the buffer
        unsafe {
            let (id, seq) = into.fields();
            self.read_next(id, seq)
        }
    }

    fn read_to_end(&mut self, into: &mut Vec<Record>) -> Result<usize> {
        let mut total = 0;

        // Read into the existing buffer
        for record in into.iter_mut() {
            if self.read_record(record)?.is_none() {
                return Ok(total);
            }
            total += 1;
        }

        // Append to the buffer
        loop {
            let mut record = Record::default();
            if self.read_record(&mut record)?.is_none() {
                return Ok(total);
            }
            into.push(record);
            total += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Report;
    use std::io::Read;
    use std::path::PathBuf;

    fn test_read_record(content: impl Read, expected: &[(&str, &str)]) -> Result<()> {
        // Record-by-record reading
        let mut reader = Reader::new(std::io::BufReader::new(content))?;
        let mut record = Record::default();
        for (id, seq) in expected {
            assert!(reader.read_record(&mut record)?.is_some());
            assert_eq!(record, (*id, *seq).try_into()?);
        }
        assert!(reader.read_record(&mut record)?.is_none());

        Ok(())
    }

    fn test_read_to_end(content: impl Read, expected: &[(&str, &str)]) -> Result<()> {
        // Read all records at once
        let mut reader = Reader::new(std::io::BufReader::new(content))?;
        let mut records = Vec::new();
        reader.read_to_end(&mut records)?;
        assert_eq!(records.len(), expected.len());
        for (record, (id, seq)) in records.iter().zip(expected.iter()) {
            assert_eq!(*record, (*id, *seq).try_into()?);
        }

        Ok(())
    }

    #[test]
    fn test_empty_fasta_reader() -> Result<()> {
        test_read_record(&[0u8; 0][..], &[])?;
        test_read_to_end(&[0u8; 0][..], &[])?;
        Ok(())
    }

    #[test]
    fn test_invalid_fasta() {
        for content in [
            " ",
            ">",
            ">id",
            ">id\nAC GT",
            ">id\nACGT ",
            ">id\nACGT\n\n",
            ">id\nACGT\nA \n",
            ">id\nACGT\n>ID\n",
            ">id\nACGT\n>ID\nACGT ",
        ] {
            // Per record
            let result = Reader::new(std::io::Cursor::new(content)).and_then(|mut x| {
                let mut record = Record::default();
                while x.read_record(&mut record)?.is_some() {}
                Ok::<(), Report>(())
            });

            assert!(result.is_err(), "Content: {:?}", content);

            // All records
            let result = Reader::new(std::io::Cursor::new(content)).and_then(|mut x| {
                let mut records = Vec::new();
                x.read_to_end(&mut records)?;
                Ok::<(), Report>(())
            });

            assert!(result.is_err(), "Content: {:?}", content);
        }
    }

    #[test]
    fn test_valid_fasta() {
        for (content, records) in [
            (">id\nACGT\n", vec![("id", "ACGT")]),
            (
                ">id\nACGT\n>id2\nACGT\n",
                vec![("id", "ACGT"), ("id2", "ACGT")],
            ),
            (
                ">id\nACGT\n>id2\nAC\n>id3\r\nGT\r\n>id4\r\nT\r\n",
                vec![("id", "ACGT"), ("id2", "AC"), ("id3", "GT"), ("id4", "T")],
            ),
            (
                ">ID\r\nACGT\r\nA\r\nGTTT\r\nA\r\n>id2\nAC\n",
                vec![("ID", "ACGTAGTTTA"), ("id2", "AC")],
            ),
        ] {
            assert!(test_read_record(content.as_bytes(), &records).is_ok());
            assert!(test_read_to_end(content.as_bytes(), &records).is_ok());
        }
    }

    #[test]
    fn test_example_fa() -> Result<()> {
        let expected = [
            (
                " My Super ЮТФ-последовательность Прямо Here   ",
                "NonUniformLinesAreAllowed",
            ),
            (
                "\tAnother UTF sequence with tabs and spaces\t",
                "AnySequenceWithoutSpacesAllowedHere",
            ),
        ];

        for fname in ["example.fa", "example.fa.gz"] {
            let file = PathBuf::from(env!("BIOBIT_RESOURCES"))
                .join("fasta")
                .join(fname);
            let read = compression::read_file(&file)?.box_read();
            test_read_record(read, &expected)?;

            let read = compression::read_file(&file)?.box_read();
            test_read_to_end(read, &expected)?;
        }

        Ok(())
    }
}
