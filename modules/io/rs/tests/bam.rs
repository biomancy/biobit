//! Small synthetic records exercise the BAM pipeline without external fixtures.

use std::collections::VecDeque;
use std::io::{self, Cursor, Error};
use std::path::PathBuf;

use higher_kinded_types::prelude::*;
use noodles::sam::alignment::io::Write as _;
use noodles::{bam, sam};

use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{Interval, Orientation};
use biobit_core_rs::source::{Core, Source, Transform};

use biobit_io_rs::bam::ReaderBuilder;
use biobit_io_rs::bam::strdeductor::deduce;
use biobit_io_rs::bam::transform::{
    BundleByOrientation, BundleMates, ExtractAlignmentSegments, ExtractPairedAlignmentSegments,
};

type Records = For!(<'borrow> = Batches<bam::Record>);
type Pairs = For!(<'borrow> = Batches<(bam::Record, bam::Record)>);

fn header() -> sam::Header {
    "@HD\tVN:1.6\tSO:coordinate\n\
     @SQ\tSN:chr1\tLN:1000\n\
     @SQ\tSN:chr2\tLN:1000\n"
        .parse()
        .unwrap()
}

fn record(line: &str) -> io::Result<bam::Record> {
    let header = header();
    let mut reader = sam::io::Reader::from(Cursor::new(line));
    let mut record = sam::alignment::RecordBuf::default();
    assert!(reader.read_record_buf(&header, &mut record)? > 0);
    let mut writer = bam::io::Writer::from(Vec::new());
    writer.write_alignment_record(&header, &record)?;

    let mut reader = bam::io::Reader::from(Cursor::new(writer.into_inner()));
    let mut record = bam::Record::default();
    assert!(reader.read_record(&mut record)? > 0);
    Ok(record)
}

// Mock source of records for testing BAM transformations
struct Batches<T> {
    batches: VecDeque<io::Result<Vec<T>>>,
    current: Vec<T>,
}

impl<T> Batches<T> {
    fn new(batches: Vec<io::Result<Vec<T>>>) -> Self {
        Self {
            batches: batches.into(),
            current: Vec::new(),
        }
    }
}

impl<T: 'static> LendingIterator for Batches<T> {
    type Item = For!(<'iter> = io::Result<&'iter mut Vec<T>>);

    fn next(&mut self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.batches.pop_front()? {
            Ok(batch) => {
                self.current = batch;
                Some(Ok(&mut self.current))
            }
            Err(error) => Some(Err(error)),
        }
    }
}

// Each fixture owns its temporary BAM and index and removes them on drop.
struct BamFile(PathBuf);

impl BamFile {
    fn new(lines: &[&str]) -> io::Result<Self> {
        let directory = std::env::temp_dir().join(format!(
            "biobit-bam-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        std::fs::create_dir(&directory)?;
        let file = Self(directory.join("sample.bam"));
        let mut writer = bam::io::Writer::new(std::fs::File::create_new(&file.0)?);
        let header = header();
        writer.write_header(&header)?;
        for line in lines {
            writer.write_alignment_record(&header, &record(line)?)?;
        }
        writer.try_finish()?;
        drop(writer);
        let bam::Index::Bai(index) = bam::fs::index(&file.0)? else {
            unreachable!("small synthetic references use BAI");
        };
        bam::bai::fs::write(file.0.with_added_extension("bai"), &index)?;
        Ok(file)
    }
}

impl Drop for BamFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.0.parent().unwrap());
    }
}

#[test]
fn extracts_zero_based_single_end_blocks() -> io::Result<()> {
    for (start, cigar, hits, expected) in [
        (1, "5M", 1, vec![(0, 5)]),
        (101, "5M", 256, vec![(100, 105)]),
        (
            1,
            "2H3S2M1I2=1X2D3M4N2M1P1M2S1H",
            u32::MAX,
            vec![(0, 5), (7, 10), (14, 17)],
        ),
    ] {
        let mapped = record(&format!(
            "r0\t0\tchr1\t{start}\t60\t{cigar}\t*\t0\t0\t*\t*\tNH:i:{hits}"
        ))?;
        let source = Batches::new(vec![Ok(vec![bam::Record::default(), mapped])]);
        let mut transform = ExtractAlignmentSegments::new(deduce::se::forward);
        let mut iter = <ExtractAlignmentSegments<_> as Transform<Records>>::transform(
            &mut transform,
            source,
            &(),
        );
        let batch = iter.next().unwrap()?;
        let expected = expected
            .into_iter()
            .map(|bounds| Interval::try_from(bounds).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(batch.intervals.at(0), expected);
        assert_eq!(batch.orientation, [Orientation::Forward]);
        assert_eq!(batch.total_hits, [hits]);
        assert!(iter.next().is_none());
    }
    Ok(())
}

#[test]
fn unions_both_mates_without_double_counting() -> io::Result<()> {
    for (left_start, left_cigar, right_start, right_cigar, expected) in [
        (1, "5M", 11, "5M", vec![(0, 5), (10, 15)]),
        (1, "10M", 6, "10M", vec![(0, 15)]),
        (1, "5M", 6, "5M", vec![(0, 10)]),
        (11, "5M", 1, "5M", vec![(0, 5), (10, 15)]),
        (1, "5M10N5M", 4, "14M", vec![(0, 20)]),
    ] {
        let left = record(&format!(
            "r0\t65\tchr1\t{left_start}\t60\t{left_cigar}\t*\t0\t0\t*\t*\tNH:i:200"
        ))?;
        let right = record(&format!(
            "r0\t145\tchr1\t{right_start}\t60\t{right_cigar}\t*\t0\t0\t*\t*\tNH:i:200"
        ))?;
        let source = Batches::new(vec![Ok(vec![(left, right)])]);
        let mut transform = ExtractPairedAlignmentSegments::new(deduce::pe::forward);
        let mut iter = <ExtractPairedAlignmentSegments<_> as Transform<Pairs>>::transform(
            &mut transform,
            source,
            &(),
        );
        let batch = iter.next().unwrap()?;
        let expected = expected
            .into_iter()
            .map(|bounds| Interval::try_from(bounds).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(batch.intervals.at(0), expected);
        assert_eq!(batch.orientation, [Orientation::Forward]);
        assert_eq!(batch.total_hits, [200]);
        assert!(iter.next().is_none());
    }
    Ok(())
}

#[test]
fn reports_bad_alignments_instead_of_panicking_or_ending_iteration() -> io::Result<()> {
    for line in [
        "r0\t0\tchr1\t0\t60\t5M\t*\t0\t0\t*\t*\tNH:i:1",
        "r0\t0\tchr1\t1\t60\t0M\t*\t0\t0\t*\t*\tNH:i:1",
        "r0\t0\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*",
        "r0\t0\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tNH:i:0",
        "r0\t0\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tNH:i:-1",
        "r0\t0\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tNH:Z:1",
    ] {
        let source = Batches::new(vec![Ok(vec![record(line)?])]);
        let mut transform = ExtractAlignmentSegments::new(deduce::se::forward);
        let mut iter = <ExtractAlignmentSegments<_> as Transform<Records>>::transform(
            &mut transform,
            source,
            &(),
        );
        assert_eq!(
            iter.next().unwrap().unwrap_err().kind(),
            io::ErrorKind::InvalidData,
            "{line}"
        );
    }

    for line in [
        "r0\t145\tchr2\t11\t60\t5M\t*\t0\t0\t*\t*\tNH:i:1",
        "r0\t129\tchr1\t11\t60\t5M\t*\t0\t0\t*\t*\tNH:i:1",
        "r0\t145\tchr1\t11\t60\t5M\t*\t0\t0\t*\t*\tNH:i:2",
    ] {
        let left = record("r0\t65\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tNH:i:1")?;
        let source = Batches::new(vec![Ok(vec![(left, record(line)?)])]);
        let mut transform = ExtractPairedAlignmentSegments::new(deduce::pe::forward);
        let mut iter = <ExtractPairedAlignmentSegments<_> as Transform<Pairs>>::transform(
            &mut transform,
            source,
            &(),
        );
        assert_eq!(
            iter.next().unwrap().unwrap_err().kind(),
            io::ErrorKind::InvalidData,
            "{line}"
        );
    }
    Ok(())
}

#[test]
fn bundles_mates_across_batches_and_preserves_hit_indices() -> io::Result<()> {
    let mut pairs = Vec::new();
    for hit_index in [0, 1, 129, 256, 70_000, u32::MAX] {
        let left = record(&format!(
            "r0\t65\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tHI:i:{hit_index}"
        ))?;
        let right = record(&format!(
            "r0\t145\tchr1\t11\t60\t5M\t*\t0\t0\t*\t*\tHI:i:{hit_index}"
        ))?;
        pairs.push((left, right));
    }
    // All pairs share a read name. Keep their right mates in flight together and
    // return the left mates in a different order to expose truncated HI cache keys.
    let source = Batches::new(vec![
        Ok(pairs.iter().rev().map(|(_, right)| right.clone()).collect()),
        Ok(vec![]),
        Ok(pairs.iter().map(|(left, _)| left.clone()).collect()),
    ]);
    let mut transform = BundleMates::new(0);
    let mut iter = <BundleMates as Transform<Records>>::transform(&mut transform, source, &());
    assert_eq!(iter.next().unwrap()?, &pairs);
    assert!(iter.next().is_none());
    Ok(())
}

#[test]
fn rejects_ambiguous_mates() -> io::Result<()> {
    fn test(line: &str) -> Error {
        let value = record(line).unwrap();
        let source = Batches::new(vec![Ok(vec![value.clone(), value])]);
        let mut transform = BundleMates::default();
        let mut iter = <BundleMates as Transform<Records>>::transform(&mut transform, source, &());
        let error = iter.next().unwrap().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{line}");
        error
    }

    for line in [
        "r0\t0\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tHI:i:1",
        "r0\t1\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tHI:i:1",
        "r0\t193\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tHI:i:1",
        "*\t65\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tHI:i:1",
        "r0\t65\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tHI:i:-1",
        "r0\t65\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tHI:Z:1",
        "r0\t65\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*",
    ] {
        test(line);
    }

    // Repeated first mates used to panic in debug builds and lose data in release.
    let duplicate = "r0\t65\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*\tHI:i:1";
    let error = test(duplicate);
    let message = error.to_string();
    for detail in ["\"r0\"", "HI=1", "mate=first", "identical=true"] {
        assert!(message.contains(detail), "{message}");
    }

    Ok(())
}

#[test]
fn filters_query_boundaries_flags_and_mapping_quality() -> eyre::Result<()> {
    let file = BamFile::new(&[
        "r0\t0\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*",
        "r3\t0\tchr1\t1\t10\t5M\t*\t0\t0\t*\t*",
        "r4\t256\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*",
        "r5\t0\tchr1\t1\t255\t5M\t*\t0\t0\t*\t*",
        "r6\t65\tchr1\t1\t60\t5M\t*\t0\t0\t*\t*",
        "r1\t0\tchr1\t6\t60\t5M\t*\t0\t0\t*\t*",
        "r2\t0\tchr2\t1\t60\t5M\t*\t0\t0\t*\t*",
    ])?;
    let contig = "chr1".to_owned();
    for (start, end, inflags, minmapq, expected) in [
        (0, 1, 0, 20, vec!["r0", "r5", "r6"]),
        (4, 5, 0, 20, vec!["r0", "r5", "r6"]),
        (5, 6, 0, 20, vec!["r1"]),
        (10, 11, 0, 20, vec![]),
        (0, 10, 64, 20, vec!["r6"]),
        (0, 10, 0, 255, vec!["r5"]),
    ] {
        let mut reader = ReaderBuilder::new(&file.0)
            .with_batch_size(1)
            .with_inflags(inflags)
            .with_exflags(256)
            .with_minmapq(minmapq)
            .build()?;
        let mut query = reader.fetch((&contig, start, end))?;
        let mut actual = Vec::new();
        while let Some(records) = query.next() {
            let records = records?;
            assert_eq!(records.len(), 1);
            actual.push(String::from_utf8_lossy(records[0].name().unwrap()).into_owned());
        }
        assert_eq!(actual, expected);
    }
    Ok(())
}

#[test]
fn reader_reports_bad_inputs_and_accepts_csi_indexes() -> eyre::Result<()> {
    assert!(
        matches!(ReaderBuilder::new("unused.bam").with_batch_size(0).build(), Err(error) if error.kind() == io::ErrorKind::InvalidInput)
    );
    let file = BamFile::new(&[])?;
    let path = &file.0;
    let mut reader = ReaderBuilder::new(path).build()?;
    let contig = "chr1".to_owned();
    for (start, end) in [(0, 0), (1, 0), (10, 5), (usize::MAX, usize::MAX)] {
        assert!(reader.fetch((&contig, start, end)).is_err());
    }
    let missing = "missing".to_owned();
    assert!(reader.fetch((&missing, 0, 1)).is_err());
    reader.with_batch_size(0);
    assert!(reader.fetch((&contig, 0, 1)).is_err());
    reader.with_batch_size(1);
    assert!(reader.fetch((&contig, 0, 1))?.next().is_none());

    // CSI is needed when a reference exceeds BAI's coordinate range.
    std::fs::remove_file(path.with_added_extension("bai"))?;
    // Clones retain the loaded index even if its file is no longer present.
    assert!(reader.clone().fetch((&contig, 0, 1))?.next().is_none());
    noodles::csi::fs::write(
        path.with_added_extension("csi"),
        &noodles::csi::Index::default(),
    )?;
    assert!(ReaderBuilder::new(path).build().is_ok());

    // Malformed input must return an error through build(), rather than panic.
    std::fs::write(path, b"not a BAM file")?;
    assert!(ReaderBuilder::new(path).build().is_err());
    Ok(())
}

#[test]
fn groups_records_by_orientation_and_propagates_errors() -> io::Result<()> {
    let forward = record("r0\t0\tchr1\t1\t60\t1M\t*\t0\t0\t*\t*")?;
    let reverse = record("r1\t16\tchr1\t1\t60\t1M\t*\t0\t0\t*\t*")?;
    let source = Batches::new(vec![
        Ok(vec![forward.clone(), reverse.clone(), forward.clone()]),
        Err(io::Error::other("boom")),
    ]);
    let mut transform = BundleByOrientation::new(deduce::se::forward);
    let mut iter =
        <BundleByOrientation<_> as Transform<Records>>::transform(&mut transform, source, &());

    for (expected_orientation, expected_records) in [
        (Orientation::Forward, vec![forward.clone(), forward]),
        (Orientation::Reverse, vec![reverse]),
    ] {
        let (orientation, records) = iter.next().unwrap()?;
        assert_eq!(orientation, expected_orientation);
        assert_eq!(records, expected_records);
    }
    assert_eq!(
        iter.next().unwrap().unwrap_err().kind(),
        io::ErrorKind::Other
    );
    assert!(iter.next().is_none());
    Ok(())
}
