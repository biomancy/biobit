use super::record::Record;
use crate::traits::WriteRecord;
use derive_getters::Dissolve;
use eyre::Result;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::Path;
use substratum_compress::{Encoder, adapter::BoxedSync, encode::Encode};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Dissolve)]
pub struct Writer<W> {
    writer: W,
    line_width: NonZeroUsize,
}

impl<W> Writer<W> {
    pub fn new(writer: W, line_width: NonZeroUsize) -> Self {
        Self { writer, line_width }
    }
}

impl Writer<()> {
    pub fn from_path(
        path: impl AsRef<Path>,
        encoder: &Encoder,
        line_width: NonZeroUsize,
    ) -> Result<Box<dyn WriteRecord<Record = Record> + Send + Sync + 'static>> {
        let file = encoder.encode(std::fs::File::create(path.as_ref())?, BoxedSync)?;
        let writer = Box::new(Writer::new(file, line_width));
        Ok(writer)
    }
}

impl<W: Write> WriteRecord for Writer<W> {
    type Record = Record;

    fn write_record(&mut self, record: &Self::Record) -> Result<()> {
        self.writer.write_all(b">")?;
        self.writer.write_all(record.id().as_bytes())?;
        self.writer.write_all(b"\n")?;

        record
            .seq()
            .chunks(self.line_width.get())
            .try_for_each(|c| -> Result<()> {
                self.writer.write_all(c)?;
                self.writer.write_all(b"\n")?;
                Ok(())
            })
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReadRecord;
    use crate::fasta::Reader;
    use std::fs::File;
    use std::io::{Cursor, Read};
    use std::path::PathBuf;
    use substratum_compress::{Decoder, decode::DecodeReadIntoRead};

    #[test]
    fn test_fasta_writer_preserves_content() -> Result<()> {
        let line_width = NonZeroUsize::new(60).unwrap();
        for fname in ["indexed.fa", "indexed.fa.bgz"] {
            let path = PathBuf::from(env!("BIOBIT_RESOURCES"))
                .join("fasta")
                .join(fname);

            let mut expected = Vec::new();
            Decoder::from_path(&path, crate::fasta::EXTENSIONS)
                .unwrap()
                .decode_read_into_read(File::open(path)?, BoxedSync)?
                .read_to_end(&mut expected)?;

            let mut records = Vec::new();
            Reader::new(Cursor::new(&expected))?.read_to_end(&mut records)?;

            let mut produced = Vec::new();
            let mut writer = Writer::new(Cursor::new(&mut produced), line_width);
            writer.write_records(&records)?;
            writer.flush()?;

            assert_eq!(String::from_utf8(produced)?, String::from_utf8(expected)?);
        }
        Ok(())
    }
}
