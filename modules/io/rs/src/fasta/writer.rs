use super::record::Record;
use crate::compression::encode;
use crate::traits::WriteRecord;
use derive_getters::Dissolve;
use eyre::Result;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::Path;

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
        config: &encode::Config,
        line_width: NonZeroUsize,
    ) -> Result<Box<dyn WriteRecord<Record = Record> + Send + Sync + 'static>> {
        let file = std::fs::File::create(path.as_ref())?;
        let boxed: Box<dyn WriteRecord<Record = Record> + Send + Sync + 'static> =
            match encode::Stream::new(file, config)? {
                encode::Stream::Raw(x) => Box::new(Writer::new(x, line_width)),
                encode::Stream::Deflate(x) => Box::new(Writer::new(x, line_width)),
                encode::Stream::Gzip(x) => Box::new(Writer::new(x, line_width)),
                encode::Stream::Bgzf(x) => Box::new(Writer::new(x, line_width)),
                encode::Stream::MultithreadedBgzf(x) => Box::new(Writer::new(x, line_width)),
            };
        Ok(boxed)
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

    fn write_records(&mut self, records: &[Self::Record]) -> Result<usize> {
        let mut count = 0;
        for record in records {
            self.write_record(record)?;
            count += 1;
        }
        Ok(count)
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::decode;
    use crate::fasta::Reader;
    use crate::ReadRecord;
    use std::io::{Cursor, Read};
    use std::path::PathBuf;

    #[test]
    fn test_fasta_reader_preserves_content() -> Result<()> {
        let line_width = NonZeroUsize::new(60).unwrap();
        for fname in ["indexed.fa", "indexed.fa.bgz"] {
            let path = PathBuf::from(env!("BIOBIT_RESOURCES"))
                .join("fasta")
                .join(fname);

            let mut expected = Vec::new();
            decode::infer_from_path(path)?.read_to_end(&mut expected)?;

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
