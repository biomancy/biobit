use crate::bed::{
    Bed12, Bed12Op, Bed3, Bed3Op, Bed4, Bed4Op, Bed5, Bed5Op, Bed6, Bed6Op, Bed8, Bed8Op, Bed9,
    Bed9Op,
};
use crate::compression::encode;
use crate::WriteRecord;
use biobit_core_rs::loc::{IntervalOp, Orientation};
use eyre::Result;
use flate2::write::{DeflateEncoder, GzEncoder};
use itertools::Itertools;
use noodles::bgzf;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Writer<W, Bed> {
    writer: W,
    _phantom: std::marker::PhantomData<Bed>,
}

impl Writer<(), ()> {
    pub fn from_path<Bed>(
        path: impl AsRef<Path>,
        compression: &encode::Config,
    ) -> Result<Box<dyn WriteRecord<Record = Bed> + Send + Sync + 'static>>
    where
        Writer<File, Bed>: WriteRecord<Record = Bed> + Send + Sync + 'static,
        Writer<DeflateEncoder<File>, Bed>: WriteRecord<Record = Bed> + Send + Sync + 'static,
        Writer<GzEncoder<File>, Bed>: WriteRecord<Record = Bed> + Send + Sync + 'static,
        Writer<bgzf::Writer<File>, Bed>: WriteRecord<Record = Bed> + Send + Sync + 'static,
        Writer<bgzf::MultithreadedWriter<File>, Bed>:
            WriteRecord<Record = Bed> + Send + Sync + 'static,
    {
        let file = File::create(path.as_ref())?;
        let slf: Box<dyn WriteRecord<Record = Bed> + Send + Sync + 'static> =
            match encode::Stream::new(file, compression)? {
                encode::Stream::Raw(x) => Box::new(Writer::new(x)),
                encode::Stream::Deflate(x) => Box::new(Writer::new(x)),
                encode::Stream::Gzip(x) => Box::new(Writer::new(x)),
                encode::Stream::Bgzf(x) => Box::new(Writer::new(x)),
                encode::Stream::MultithreadedBgzf(x) => Box::new(Writer::new(x)),
            };
        Ok(slf)
    }
}

impl<W: Write, Bed> Writer<W, Bed> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _phantom: std::marker::PhantomData,
        }
    }
}

macro_rules! impl_write_record {
    ($record:ident, $([$field:expr],)+) => {};
    ($record:ident, $([$field:expr],)+ $Bed:ident, $template:literal, $($tail:tt)*) => {
        impl<W: Write> WriteRecord for Writer<W, $Bed> {
            type Record = $Bed;

            fn write_record(&mut self, $record: &Self::Record) -> Result<()> {
                writeln!(self.writer, $template, $($field,)*)?;
                Ok(())
            }

            fn flush(&mut self) -> Result<()> {
                self.writer.flush()?;
                Ok(())
            }
        }

        impl_write_record!($record, $([$field],)+ $($tail)*);
    };
}

impl_write_record!(
    record,
    // Bed3
    [record.seqid()],
    [record.interval().start()],
    [record.interval().end()],
    Bed3,
    "{}\t{}\t{}",
    // Bed4
    [record.name()],
    Bed4,
    "{}\t{}\t{}\t{}",
    // Bed5
    [record.score()],
    Bed5,
    "{}\t{}\t{}\t{}\t{}",
    // Bed6
    [match record.orientation() {
        Orientation::Forward => "+",
        Orientation::Reverse => "-",
        Orientation::Dual => ".",
    }],
    Bed6,
    "{}\t{}\t{}\t{}\t{}\t{}",
    // Bed8
    [record.thick().start()],
    [record.thick().end()],
    Bed8,
    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
    // Bed9
    [record.rgb().0],
    [record.rgb().1],
    [record.rgb().2],
    Bed9,
    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{},{},{}",
    // Bed12
    [record.blocks().len()],
    [Itertools::join(
        &mut record.blocks().iter().map(|x| x.len()),
        ","
    )],
    [Itertools::join(
        &mut record.blocks().iter().map(|x| x.start()),
        ","
    )],
    Bed12,
    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{},{},{}\t{}\t{}\t{}",
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bed::Reader;
    use crate::compression::decode;
    use crate::ReadRecord;
    use std::io::{Cursor, Read};
    use std::path::PathBuf;

    #[test]
    fn test_bed12_writer_preserves_content() -> Result<()> {
        for fname in ["example.bed", "example.bed.gz"] {
            let path = PathBuf::from(env!("BIOBIT_RESOURCES"))
                .join("bed")
                .join(fname);

            let mut expected = Vec::new();
            decode::infer_from_path(path)?.read_to_end(&mut expected)?;

            let mut records = Vec::new();
            Reader::<_, Bed12>::new(Cursor::new(&expected))?.read_to_end(&mut records)?;

            let mut produced = Vec::new();
            let mut writer = Writer::<_, Bed12>::new(Cursor::new(&mut produced));
            writer.write_records(&records)?;

            assert_eq!(String::from_utf8(produced)?, String::from_utf8(expected)?);
        }

        Ok(())
    }
}
