use super::record::*;
use crate::ReadRecord;
use crate::compression::decode;
use biobit_core_rs::loc::{Interval, Orientation};
use eyre::OptionExt;
use eyre::{Context, Result, bail, ensure};
use flate2::read::{DeflateDecoder, MultiGzDecoder};
use noodles::bgzf;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::path::Path;

pub mod parse {
    use super::*;

    pub fn seqid<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<String> {
        let seqid = parts.next().ok_or_eyre("Missing BED seqid")?;
        Ok(seqid.to_owned())
    }

    pub fn interval<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<Interval<u64>> {
        let start = parts.next().ok_or_eyre("Missing BED start")?;
        let end = parts.next().ok_or_eyre("Missing BED end")?;

        let (start, end) = match (start.parse::<u64>(), end.parse::<u64>()) {
            (Ok(start), Ok(end)) => (start, end),
            _ => bail!("Invalid BED interval"),
        };
        let interval = Interval::new(start, end).wrap_err("Invalid BED interval")?;
        Ok(interval)
    }

    pub fn name<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<String> {
        let name = parts.next().ok_or_eyre("Missing BED name")?;
        Ok(name.to_owned())
    }

    pub fn score<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<u16> {
        let score = parts.next().ok_or_eyre("Missing BED score")?;
        let score = score.parse::<u16>().wrap_err("Invalid BED score")?;
        Ok(score)
    }

    pub fn orientation<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<Orientation> {
        let orientation = parts.next().ok_or_eyre("Missing BED orientation")?;
        let orientation = match orientation {
            "+" => Orientation::Forward,
            "-" => Orientation::Reverse,
            "." => Orientation::Dual,
            _ => bail!("Invalid BED strand"),
        };
        Ok(orientation)
    }

    pub fn thick<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<Interval<u64>> {
        let start = parts.next().ok_or_eyre("Missing BED thickStart")?;
        let start = start.parse::<u64>().wrap_err("Invalid BED thickStart")?;

        let end = parts.next().ok_or_eyre("Missing BED thickEnd")?;
        let end = end.parse::<u64>().wrap_err("Invalid BED thickEnd")?;

        Interval::new(start, end).wrap_err("Invalid BED thick interval")
    }

    pub fn rgb<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<(u8, u8, u8)> {
        let rgb = parts.next().ok_or_eyre("Missing BED rgb")?;

        if rgb == "0" {
            let rgb = (0, 0, 0);
            return Ok(rgb);
        }

        let mut parts = rgb.split(',');
        let r = parts
            .next()
            .ok_or_else(|| eyre::eyre!("Missing BED itemRgb red value"))?
            .parse::<u8>()
            .wrap_err_with(|| "Invalid BED itemRgb red value")?;
        let g = parts
            .next()
            .ok_or_else(|| eyre::eyre!("Missing BED itemRgb green value"))?
            .parse::<u8>()
            .wrap_err_with(|| "Invalid BED itemRgb green value")?;
        let b = parts
            .next()
            .ok_or_else(|| eyre::eyre!("Missing BED itemRgb blue value"))?
            .parse::<u8>()
            .wrap_err_with(|| "Invalid BED itemRgb blue value")?;

        ensure!(
            parts.next().is_none(),
            "BED itemRgb must be either 0 or have three comma-separated values"
        );
        let rgb = (r, g, b);
        Ok(rgb)
    }

    pub fn blocks<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<Vec<Interval<u64>>> {
        let count = parts.next().ok_or_eyre("Missing BED blockCount")?;
        let count = count.parse::<u64>().wrap_err("Invalid BED blockCount")?;
        ensure!(count > 0, "BED blockCount must be greater than 0");

        let sizes = parts.next().ok_or_eyre("Missing BED blockSizes")?;
        let starts = parts.next().ok_or_eyre("Missing BED blockStarts")?;

        // Trim the trailing comma if it exists in sizes/starts (allowed by the BED specification)
        let mut sizes = sizes.strip_suffix(',').unwrap_or(sizes).split(',');
        let mut starts = starts.strip_suffix(',').unwrap_or(starts).split(',');

        let mut results = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let (size, start) = match (sizes.next(), starts.next()) {
                (Some(size), Some(start)) => (size, start),
                _ => bail!("BED blockCount does not match the number of blocks in the record"),
            };

            let size = size.parse::<u64>().wrap_err("Invalid BED blockSizes")?;
            let start = start.parse::<u64>().wrap_err("Invalid BED blockStarts")?;

            let block = Interval::new(start, start + size).wrap_err("Invalid BED blocks")?;
            results.push(block);
        }

        ensure!(
            sizes.next().is_none() && starts.next().is_none(),
            "BED blockCount does not match the number of blocks in the record"
        );

        Ok(results)
    }

    pub fn bed3<'a>(parts: &mut impl Iterator<Item = &'a str>, into: &mut Bed3) -> Result<()> {
        into.set(Some(seqid(parts)?), Some(interval(parts)?))?;
        Ok(())
    }

    pub fn bed4<'a>(parts: &mut impl Iterator<Item = &'a str>, into: &mut Bed4) -> Result<()> {
        into.set(
            Some(seqid(parts)?),
            Some(interval(parts)?),
            Some(name(parts)?),
        )?;
        Ok(())
    }

    pub fn bed5<'a>(parts: &mut impl Iterator<Item = &'a str>, into: &mut Bed5) -> Result<()> {
        into.set(
            Some(seqid(parts)?),
            Some(interval(parts)?),
            Some(name(parts)?),
            Some(score(parts)?),
        )?;
        Ok(())
    }

    pub fn bed6<'a>(parts: &mut impl Iterator<Item = &'a str>, into: &mut Bed6) -> Result<()> {
        into.set(
            Some(seqid(parts)?),
            Some(interval(parts)?),
            Some(name(parts)?),
            Some(score(parts)?),
            Some(orientation(parts)?),
        )?;
        Ok(())
    }

    pub fn bed8<'a>(parts: &mut impl Iterator<Item = &'a str>, into: &mut Bed8) -> Result<()> {
        into.set(
            Some(seqid(parts)?),
            Some(interval(parts)?),
            Some(name(parts)?),
            Some(score(parts)?),
            Some(orientation(parts)?),
            Some(thick(parts)?),
        )?;
        Ok(())
    }

    pub fn bed9<'a>(parts: &mut impl Iterator<Item = &'a str>, into: &mut Bed9) -> Result<()> {
        into.set(
            Some(seqid(parts)?),
            Some(interval(parts)?),
            Some(name(parts)?),
            Some(score(parts)?),
            Some(orientation(parts)?),
            Some(thick(parts)?),
            Some(rgb(parts)?),
        )?;
        Ok(())
    }

    pub fn bed12<'a>(parts: &mut impl Iterator<Item = &'a str>, into: &mut Bed12) -> Result<()> {
        into.set(
            Some(seqid(parts)?),
            Some(interval(parts)?),
            Some(name(parts)?),
            Some(score(parts)?),
            Some(orientation(parts)?),
            Some(thick(parts)?),
            Some(rgb(parts)?),
            Some(blocks(parts)?),
        )?;

        Ok(())
    }
}

pub struct Reader<R, Bed> {
    reader: R,
    buffer: String,
    _phantom_data: PhantomData<Bed>,
}

impl<R, Bed> Reader<R, Bed> {
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            reader,
            buffer: String::new(),
            _phantom_data: Default::default(),
        })
    }
}

impl Reader<(), ()> {
    /// Create a new BED reader from the given file path.
    /// The compression is automatically detected based on the file extension and the internal file signature.
    pub fn from_path<Bed>(
        path: impl AsRef<Path>,
        compression: &decode::Config,
    ) -> Result<Box<dyn ReadRecord<Record = Bed> + Send + Sync + 'static>>
    where
        Bed: Send + Sync + 'static,
        Reader<BufReader<File>, Bed>: ReadRecord<Record = Bed>,
        Reader<BufReader<MultiGzDecoder<File>>, Bed>: ReadRecord<Record = Bed>,
        Reader<BufReader<DeflateDecoder<File>>, Bed>: ReadRecord<Record = Bed>,
        Reader<BufReader<bgzf::Reader<File>>, Bed>: ReadRecord<Record = Bed>,
        Reader<BufReader<bgzf::MultithreadedReader<File>>, Bed>: ReadRecord<Record = Bed>,
    {
        let file = File::open(path.as_ref())?;
        let slf: Box<dyn ReadRecord<Record = Bed> + Send + Sync + 'static> =
            match decode::Stream::new(file, compression)? {
                decode::Stream::Raw(x) => Box::new(Reader::<_, Bed>::new(BufReader::new(x))?),
                decode::Stream::Deflate(x) => Box::new(Reader::<_, Bed>::new(BufReader::new(x))?),
                decode::Stream::Gzip(x) => Box::new(Reader::<_, Bed>::new(BufReader::new(x))?),
                decode::Stream::Bgzf(x) => Box::new(Reader::<_, Bed>::new(BufReader::new(x))?),
                decode::Stream::MultithreadedBgzf(x) => {
                    Box::new(Reader::<_, Bed>::new(BufReader::new(x))?)
                }
            };
        Ok(slf)
    }
}

macro_rules! impl_reader {
    () => {};
    (($Bed:ident, $parsing:expr), $($tail:tt,)*) => {
        impl_reader!($($tail,)*);

        impl<R: BufRead> ReadRecord for Reader<R, $Bed> {
            type Record = $Bed;

            fn read_record(&mut self, into: &mut $Bed) -> Result<bool> {
                self.buffer.clear();
                if self.reader.read_line(&mut self.buffer)? == 0 {
                    return Ok(false);
                }

                let mut parts = self
                    .buffer
                    .trim_end_matches(|c| c == '\n' || c == '\r')
                    .split('\t');
                $parsing(&mut parts, into)
                    .wrap_err_with(|| format!("Failed to parse BED record: {}", self.buffer))?;
                ensure!(
                    parts.next().is_none(),
                    "BED record has too many fields: {}",
                    self.buffer
                );
                Ok(true)
            }

            fn read_records(&mut self, into: &mut [$Bed]) -> Result<usize> {
                let mut total = 0;
                for buf in into.iter_mut() {
                    if !self.read_record(buf)? {
                        return Ok(total);
                    }
                    total += 1;
                }
                Ok(total)
            }

            fn read_to_end(&mut self, into: &mut Vec<$Bed>) -> Result<usize> {
                let mut total = 0;

                // Read into the existing buffer
                for record in into.iter_mut() {
                    if !self.read_record(record)? {
                        return Ok(total);
                    }
                    total += 1;
                }

                // Append to the buffer
                loop {
                    let mut record = $Bed::default();
                    if !self.read_record(&mut record)? {
                        return Ok(total);
                    }
                    into.push(record);
                    total += 1;
                }
            }
        }
    };
}

impl_reader!(
    (Bed3, parse::bed3),
    (Bed4, parse::bed4),
    (Bed5, parse::bed5),
    (Bed6, parse::bed6),
    (Bed8, parse::bed8),
    (Bed9, parse::bed9),
    (Bed12, parse::bed12),
);

#[cfg(test)]
mod test {
    use super::*;
    use itertools::Itertools;
    use std::io::{Cursor, Read};
    use std::path::PathBuf;

    fn test_reader(parts: impl Read, expected: &[Bed12]) -> Result<()> {
        let lines = BufReader::new(parts)
            .lines()
            .collect::<std::io::Result<Vec<String>>>()?;

        macro_rules! test_all_bed_impl {
            () => {};
            (($BedX:ident, $fields:expr), $($tail:tt,)*) => {
                let _lines = lines
                    .iter()
                    .map(|x| x.split("\t").collect_vec()[..$fields].to_owned().join("\t"))
                    .collect::<Vec<String>>()
                    .join("\n");

                // Record-by-record
                let mut reader = Reader::<_, $BedX>::new(Cursor::new(_lines.clone()))?;
                let mut record = $BedX::default();
                for expected in expected.iter() {
                    assert!(reader.read_record(&mut record)?);
                    assert_eq!(&record, &expected.clone().into());
                }
                assert!(!reader.read_record(&mut record)?);

                // Read to end
                let mut reader = Reader::<_, $BedX>::new(Cursor::new(_lines))?;
                let mut records = Vec::<$BedX>::new();
                reader.read_to_end(&mut records)?;
                assert_eq!(records.len(), expected.len());
                for (record, expected) in records.iter().zip(expected.iter()) {
                    assert_eq!(record, &expected.clone().into());
                }
                assert_eq!(reader.read_to_end(&mut records)?, 0);

                test_all_bed_impl!($($tail,)*);
            };
        }

        test_all_bed_impl!(
            (Bed3, 3),
            (Bed4, 4),
            (Bed5, 5),
            (Bed6, 6),
            (Bed8, 8),
            (Bed9, 9),
            (Bed12, 12),
        );
        Ok(())
    }

    #[test]
    fn test_empty_bed() -> Result<()> {
        test_reader(&[0u8; 0][..], &[])?;
        Ok(())
    }

    #[test]
    fn test_valid_bed_parsing() -> Result<()> {
        let content = "\
        chr1\t110\t200\tname\t1000\t+\t150\t175\t0\t2\t10,50\t0,40\
        ";
        let expected = vec![Bed12::new(
            "chr1".to_owned(),
            Interval::new(110, 200).unwrap(),
            "name".to_owned(),
            1000,
            Orientation::Forward,
            Interval::new(150, 175).unwrap(),
            (0, 0, 0),
            vec![
                Interval::new(0, 10).unwrap(),
                Interval::new(40, 90).unwrap(),
            ],
        )?];
        test_reader(content.as_bytes(), &expected)?;

        Ok(())
    }

    #[test]
    fn test_example_bed() -> Result<()> {
        let mut expected = Vec::new();
        for fields in [
            (
                "12",
                (100171448, 100171534),
                "1064+]",
                0,
                Orientation::Forward,
                (100171448, 100171534),
                (0, 0, 0),
                vec![(0, 86)],
            ),
            (
                "13",
                (31643773, 31646400),
                "204+]",
                13,
                Orientation::Dual,
                (31643773, 31646400),
                (0, 0, 255),
                vec![(0, 250), (2185, 2627)],
            ),
            (
                "17",
                (38362989, 38379729),
                "668+]",
                98,
                Orientation::Dual,
                (38362989, 38379729),
                (0, 0, 0),
                vec![
                    (0, 166),
                    (1894, 1981),
                    (3152, 3398),
                    (10024, 10141),
                    (12682, 12807),
                    (13258, 13375),
                    (15678, 16740),
                ],
            ),
            (
                "6",
                (137457714, 137460096),
                "129 -]",
                1000,
                Orientation::Reverse,
                (137457714, 137460096),
                (0, 255, 0),
                vec![(0, 2382)],
            ),
        ] {
            expected.push(Bed12::new(
                fields.0.to_string(),                   // seqid
                Interval::new(fields.1.0, fields.1.1)?, // interval
                fields.2.to_string(),                   // name
                fields.3,                               // score
                fields.4,                               // orientation
                Interval::new(fields.5.0, fields.5.1)?, // thick
                fields.6,                               // rgb
                fields
                    .7
                    .iter()
                    .map(|(start, end)| Interval::new(*start, *end))
                    .collect::<Result<Vec<_>>>()?,
            )?);
        }

        for fname in ["example.bed", "example.bed.gz"] {
            let file = PathBuf::from(env!("BIOBIT_RESOURCES"))
                .join("bed")
                .join(fname);

            // Directly test the file
            let mut buffer: Vec<Bed12> = Vec::new();
            Reader::from_path(&file, &decode::Config::infer_from_path(&file))?
                .read_to_end(&mut buffer)?;
            assert_eq!(buffer, expected);

            // Test all Bed combinations
            let read = decode::infer_from_path(&file)?.boxed();
            test_reader(read, &expected)?;
        }

        Ok(())
    }
}
