use crate::WriteRecord;
use crate::bed::{
    Bed3, Bed3Op, Bed4, Bed4Op, Bed5, Bed5Op, Bed6, Bed6Op, Bed8, Bed8Op, Bed9, Bed9Op, Bed12,
    Bed12Op,
};
use biobit_core_rs::loc::{IntervalOp, Orientation};
use eyre::Result;
use itertools::Itertools;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use substratum_compress::{Encoder, adapter::BoxedSync, encode::Encode};

pub struct Writer<W, Bed> {
    writer: W,
    _phantom: std::marker::PhantomData<Bed>,
}

impl Writer<(), ()> {
    pub fn from_path<Bed>(
        path: impl AsRef<Path>,
        compression: &Encoder,
    ) -> Result<Box<dyn WriteRecord<Record = Bed> + Send + Sync + 'static>>
    where
        Bed: Send + Sync + 'static,
        Writer<Box<dyn Write + Send + Sync>, Bed>: WriteRecord<Record = Bed>,
    {
        let boxed = compression.encode(File::create(path.as_ref())?, BoxedSync)?;
        let slf = Box::new(Writer::new(boxed));
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
    use crate::ReadRecord;
    use crate::bed::Reader;
    use std::io::{Cursor, Read};
    use std::path::PathBuf;
    use substratum_compress::{Decoder, adapter::BoxedSync, decode::DecodeReadIntoRead};

    #[test]
    fn test_bed12_writer_preserves_content() -> Result<()> {
        for fname in ["example.bed", "example.bed.gz"] {
            let path = PathBuf::from(env!("BIOBIT_RESOURCES"))
                .join("bed")
                .join(fname);

            let mut expected = Vec::new();
            Decoder::from_path(&path, crate::bed::EXTENSIONS)?
                .decode_read_into_read(File::open(path)?, BoxedSync)?
                .read_to_end(&mut expected)?;

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
