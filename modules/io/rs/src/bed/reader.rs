use super::record::*;
use crate::ReadRecord;
use biobit_core_rs::loc::{Interval, Orientation};
use eyre::OptionExt;
use eyre::{Context, Result, bail, ensure};
use memchr::{memchr2, memrchr2};
use std::fs::File;
use std::io::BufRead;
use std::marker::PhantomData;
use std::path::Path;
use substratum_compress::{Decoder, adapter::BoxedSync, decode::DecodeReadIntoBufRead};

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

        // The count is untrusted: start with at most 24 and grow as actual blocks are parsed.
        let mut results = Vec::with_capacity(count.min(24) as usize);
        for _ in 0..count {
            let (size, start) = match (sizes.next(), starts.next()) {
                (Some(size), Some(start)) => (size, start),
                _ => bail!("BED blockCount does not match the number of blocks in the record"),
            };

            let size = size.parse::<u64>().wrap_err("Invalid BED blockSizes")?;
            let start = start.parse::<u64>().wrap_err("Invalid BED blockStarts")?;

            let end = start
                .checked_add(size)
                .ok_or_eyre("BED block end exceeds u64")?;
            let block = Interval::new(start, end).wrap_err("Invalid BED blocks")?;
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

/// Reads BED records, skipping comments and blank lines. Accepts LF, CRLF, or CR line endings and horizontal whitespace separators
pub struct Reader<R, Bed> {
    reader: R,
    buffer: Vec<u8>,
    consume_lf: bool,
    _phantom_data: PhantomData<Bed>,
}

impl<R, Bed> Reader<R, Bed> {
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            reader,
            buffer: Vec::new(),
            consume_lf: false,
            _phantom_data: Default::default(),
        })
    }
}

impl<R: BufRead, Bed> Reader<R, Bed> {
    fn read_line(&mut self) -> Result<bool> {
        self.buffer.clear();

        let mut available = loop {
            match self.reader.fill_buf() {
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                result => break result?,
            }
        };

        // Resolve a CR at the previous buffer's end once, keeping this slice
        // for the line scan. Consume any skipped LF together with its remainder.
        let mut used = usize::from(self.consume_lf && available.first() == Some(&b'\n'));
        self.consume_lf = false;
        if available.is_empty() {
            return Ok(false);
        }
        available = &available[used..];

        loop {
            // Find the end of the line, consuming the buffer up to its end.
            if let Some(end) = memchr2(b'\r', b'\n', available) {
                self.buffer.extend_from_slice(&available[..end]);
                let cr = available[end] == b'\r';
                let crlf = cr && available.get(end + 1) == Some(&b'\n');
                self.consume_lf = cr && end + 1 == available.len();
                self.reader.consume(used + end + 1 + usize::from(crlf));
                return Ok(true);
            }

            // The line is too long to fit in the buffer, so consume it all and try again.
            self.buffer.extend_from_slice(available);
            used += available.len();
            self.reader.consume(used);
            used = 0;

            available = loop {
                match self.reader.fill_buf() {
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    result => break result?,
                }
            };
            if available.is_empty() {
                return Ok(!self.buffer.is_empty());
            }
        }
    }
}

impl Reader<(), ()> {
    /// Create a new BED reader from the given file path and the supplied decoder.
    pub fn from_path<Bed>(
        path: impl AsRef<Path>,
        decoder: &Decoder,
    ) -> Result<Box<dyn ReadRecord<Record = Bed> + Send + Sync + 'static>>
    where
        Bed: Send + Sync + 'static,
        Reader<Box<dyn BufRead + Send + Sync + 'static>, Bed>:
            ReadRecord<Record = Bed> + Send + Sync + 'static,
    {
        let file = File::open(path.as_ref())?;
        let src = decoder.decode_read_into_bufread(file, BoxedSync)?;
        let slf = Box::new(Reader::<_, Bed>::new(src)?);
        Ok(slf)
    }
}

fn split_fields<const N: usize>(mut remaining: &str) -> Result<[&str; N]> {
    let mut fields = [""; N];
    let prefix_len = if N == 3 { 2 } else { 3 };

    // The first field has no preceding separator to skip.
    let separator =
        memchr2(b' ', b'\t', remaining.as_bytes()).ok_or_eyre("BED record has too few fields")?;
    fields[0] = &remaining[..separator];
    let mut strict_tab_sep = remaining.as_bytes()[separator] == b'\t';
    remaining = &remaining[separator + 1..];

    // Read the remaining coordinate fields from the left.
    for field in &mut fields[1..prefix_len] {
        let trimmed = remaining.trim_start_matches([' ', '\t']);
        strict_tab_sep &= trimmed.len() == remaining.len();
        let separator =
            memchr2(b' ', b'\t', trimmed.as_bytes()).ok_or_eyre("BED record has too few fields")?;
        *field = &trimmed[..separator];
        strict_tab_sep &= trimmed.as_bytes()[separator] == b'\t';
        remaining = &trimmed[separator + 1..];
    }

    if N == 3 {
        let chrom_end = remaining.trim_start_matches([' ', '\t']);
        ensure!(
            !chrom_end.contains([' ', '\t']),
            "BED record has too many fields"
        );
        fields[2] = chrom_end;
        return Ok(fields);
    }

    // Read suffix fields from the right, leaving the name untouched.
    if N > 4 {
        let separator = memrchr2(b' ', b'\t', remaining.as_bytes())
            .ok_or_eyre("BED record has too few fields")?;
        fields[N - 1] = &remaining[separator + 1..];
        strict_tab_sep &= remaining.as_bytes()[separator] == b'\t';
        remaining = &remaining[..separator];

        for field in fields[4..N - 1].iter_mut().rev() {
            let trimmed = remaining.trim_end_matches([' ', '\t']);
            strict_tab_sep &= trimmed.len() == remaining.len();
            let separator = memrchr2(b' ', b'\t', trimmed.as_bytes())
                .ok_or_eyre("BED record has too few fields")?;
            *field = &trimmed[separator + 1..];
            strict_tab_sep &= trimmed.as_bytes()[separator] == b'\t';
            remaining = &trimmed[..separator];
        }

        // Extra tabs at either name boundary identify whitespace separator runs.
        if !strict_tab_sep
            || remaining.trim_start_matches(' ').starts_with('\t')
            || remaining.trim_end_matches(' ').ends_with('\t')
        {
            strict_tab_sep = false;
            remaining = remaining.trim_start_matches([' ', '\t']);
            remaining = remaining.trim_end_matches([' ', '\t']);
        }
    } else if !strict_tab_sep || remaining.trim_start_matches(' ').starts_with('\t') {
        // BED4 has no suffix - only look at separators before the name
        strict_tab_sep = false;
        remaining = remaining.trim_start_matches([' ', '\t']);
    }

    ensure!(!remaining.contains('\t'), "BED name must not contain tabs");
    ensure!(
        strict_tab_sep || !remaining.contains(' '),
        "BED names containing spaces require single-tab separators"
    );
    fields[3] = remaining;
    Ok(fields)
}

macro_rules! impl_reader {
    () => {};
    (($Bed:ident, $fields:literal, $parsing:expr), $($tail:tt,)*) => {
        impl_reader!($($tail,)*);

        impl<R: BufRead> ReadRecord for Reader<R, $Bed> {
            type Record = $Bed;

            fn read_record(&mut self, into: &mut $Bed) -> Result<bool> {
                // Comments and blank lines do not contribute fields or separators.
                loop {
                    if !self.read_line()? {
                        return Ok(false);
                    }
                    if self.buffer.starts_with(b"#")
                        || self.buffer.iter().all(|c| matches!(c, b' ' | b'\t'))
                    {
                        continue;
                    }
                    break;
                }

                std::str::from_utf8(&self.buffer)
                    .wrap_err("Invalid BED text")
                    .and_then(split_fields::<$fields>)
                    .and_then(|fields| $parsing(&mut fields.into_iter(), into))
                    .wrap_err_with(|| format!(
                        "Failed to parse BED record: \"{}\"",
                        self.buffer.escape_ascii()
                    ))?;
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
    (Bed3, 3, parse::bed3),
    (Bed4, 4, parse::bed4),
    (Bed5, 5, parse::bed5),
    (Bed6, 6, parse::bed6),
    (Bed8, 8, parse::bed8),
    (Bed9, 9, parse::bed9),
    (Bed12, 12, parse::bed12),
);

#[cfg(test)]
mod test {
    use super::*;
    use itertools::Itertools;
    use std::io::{BufReader, Cursor, Read};
    use std::path::PathBuf;
    use substratum_compress::decode::DecodeReadIntoRead;

    fn test_reader(parts: impl Read, expected: &[Bed12]) -> Result<()> {
        let lines = BufReader::new(parts)
            .lines()
            .collect::<std::io::Result<Vec<String>>>()?;

        macro_rules! test_all_bed_impl {
            () => {};
            (($BedX:ident, $fields:expr), $($tail:tt,)*) => {
                let content = lines
                    .iter()
                    .map(|line| line.split('\t').take($fields).join("\t"))
                    .join("\n");

                // Record-by-record
                let mut reader = Reader::<_, $BedX>::new(Cursor::new(&content))?;
                let mut record = $BedX::default();
                for expected in expected.iter() {
                    assert!(reader.read_record(&mut record)?);
                    assert_eq!(&record, &expected.clone().into());
                }
                assert!(!reader.read_record(&mut record)?);

                // Read to end
                let mut reader = Reader::<_, $BedX>::new(Cursor::new(content))?;
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
    fn test_valid_bed_parsing() -> Result<()> {
        test_reader("".as_bytes(), &[])?;
        for name in ["name", " name  with spaces ", " "] {
            let content = format!("chr1\t110\t200\t{name}\t1000\t+\t150\t175\t0\t2\t10,50\t0,40");
            let expected = Bed12::new(
                "chr1".into(),
                Interval::new(110, 200)?,
                name.into(),
                1000,
                Orientation::Forward,
                Interval::new(150, 175)?,
                (0, 0, 0),
                vec![Interval::new(0, 10)?, Interval::new(40, 90)?],
            )?;
            test_reader(content.as_bytes(), &[expected])?;
        }

        // Mix separator styles within one file, including names with spaces.
        let rows = [
            ("chr1  10  20  first", "first"),
            (
                "chr1\t10\t20\t a name  with spaces ",
                " a name  with spaces ",
            ),
            ("chr1\t\t10\t\t20\t\tthird", "third"),
            ("chr1\t10\t20\tanother name", "another name"),
            ("chr1 \t 10 \t 20 \t fifth", "fifth"),
        ];
        let expected = rows
            .iter()
            .map(|(_, name)| Bed4::new("chr1".into(), Interval::new(10, 20)?, (*name).into()))
            .collect::<Result<Vec<_>>>()?;
        for ending in ["\n", "\r\n", "\r"] {
            let content = rows
                .iter()
                .map(|(row, _)| *row)
                .join(&format!("{ending}# comment{ending} \t{ending}"));
            let content = format!("# head{ending} \t{ending}{content}{ending}\t {ending}# tail");
            // Small buffers force CRLF and records across refill boundaries.
            for capacity in [1, 7, 256] {
                let input = BufReader::with_capacity(capacity, Cursor::new(&content));
                let mut reader = Reader::<_, Bed4>::new(input)?;
                let mut records = Vec::new();
                assert_eq!(reader.read_to_end(&mut records)?, expected.len());
                assert_eq!(records, expected);
                assert_eq!(reader.read_to_end(&mut records)?, 0);
            }
        }

        // Exercise separator runs in both the coordinate and suffix fields.
        for separator in ["\t", "  ", "\t\t", " \t "] {
            let coordinates = ["chr1", "10", "20"].join(separator);
            let mut reader = Reader::<_, Bed3>::new(Cursor::new(coordinates))?;
            let mut record = Bed3::default();
            assert!(reader.read_record(&mut record)?);
            assert_eq!(record, Bed3::new("chr1".into(), Interval::new(10, 20)?)?);

            let row = ["chr1", "10", "20", "name", "42", "+"].join(separator);
            let mut reader = Reader::<_, Bed6>::new(Cursor::new(row))?;
            let mut record = Bed6::default();
            assert!(reader.read_record(&mut record)?);
            assert_eq!(
                record,
                Bed6::new(
                    "chr1".into(),
                    Interval::new(10, 20)?,
                    "name".into(),
                    42,
                    Orientation::Forward,
                )?
            );
        }

        // Cover separator runs beside the name and preserve a space-only name.
        for (row, name) in [
            ("chr1\t10\t20\t \tname\t42", "name"),
            ("chr1\t10\t20\tname\t \t42", "name"),
            ("chr1\t10\t20\t \tname\t \t42", "name"),
            ("chr1\t10\t20\t\tname \t42", "name"),
            ("chr1\t10\t20\t \tname \t42", "name"),
            ("chr1\t10\t20\t name 42", "name"),
            ("chr1\t10\t20\t \t42", " "),
        ] {
            let mut reader = Reader::<_, Bed5>::new(Cursor::new(row))?;
            let mut record = Bed5::default();
            assert!(reader.read_record(&mut record)?);
            assert_eq!(
                record,
                Bed5::new("chr1".into(), Interval::new(10, 20)?, name.into(), 42)?
            );
        }
        Ok(())
    }

    #[test]
    fn test_invalid_bed_parsing() -> Result<()> {
        for row in [
            " chr1\t10\t20\tname\t42\t+",
            "\tchr1\t10\t20\tname\t42\t+",
            "chr1\t10\t20",
            "chr1\t10\t20\t\t42\t+",
            "chr1\t10\t20\tname\t42",
            "chr1\t10\t20\tname\textra\t42\t+",
            "chr1\t10\t20\tname\t42\t+\textra",
            "chr1\t10\t20\tname\t42\t+ ",
            "chr1\t10\t20\tname\t42\t+\t",
            "chr1\t10\t20\tname\tnot-a-score\t+",
            "chr1 10 20 name with spaces 42 +",
            "chr1\t10\t20\tname with spaces 42 +",
            "chr1\t10\t20\t \tname with spaces\t42\t+",
            "chr1\t10\t20\tname with spaces\t \t42\t+",
        ] {
            let mut reader = Reader::<_, Bed6>::new(Cursor::new(row))?;
            let error = reader.read_record(&mut Bed6::default()).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains(&row.as_bytes().escape_ascii().to_string()),
                "{error:#}"
            );
        }

        // Bed3-specific cases
        for row in [
            "chr1\t10",
            " chr1\t10\t20",
            "chr1\t10\t20 ",
            "chr1\t10\t20\textra",
        ] {
            let mut reader = Reader::<_, Bed3>::new(Cursor::new(row))?;
            assert!(reader.read_record(&mut Bed3::default()).is_err(), "{row:?}");
        }

        // Bed4-specific cases
        for row in ["chr1 10 20 name ", "chr1\t10\t20\tname\t"] {
            let mut reader = Reader::<_, Bed4>::new(Cursor::new(row))?;
            assert!(reader.read_record(&mut Bed4::default()).is_err(), "{row:?}");
        }

        let mut reader = Reader::<_, Bed4>::new(Cursor::new("chr1\t10\t20\tname\textra"))?;
        let error = reader.read_record(&mut Bed4::default()).unwrap_err();
        assert!(format!("{error:#}").contains("BED name must not contain tabs"));

        // xff should be rejected in the BED name field
        let row = b"chr1\t10\t20\tname\xff\t42\t+";
        let mut reader = Reader::<_, Bed6>::new(Cursor::new(row))?;
        let error = reader.read_record(&mut Bed6::default()).unwrap_err();
        assert!(
            error.to_string().contains(r"chr1\t10\t20\tname\xff\t42\t+"),
            "{error:#}"
        );
        assert!(format!("{error:#}").contains("Invalid BED text"));
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
            let decoder = Decoder::from_path(&file, crate::bed::EXTENSIONS).unwrap();

            Reader::from_path(&file, &decoder)?.read_to_end(&mut buffer)?;
            assert_eq!(buffer, expected);

            // Test all Bed combinations
            let read = decoder.decode_read_into_read(File::open(&file)?, BoxedSync)?;
            test_reader(read, &expected)?;
        }

        Ok(())
    }
}
