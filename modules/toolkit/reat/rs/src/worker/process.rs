use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::num::PrimUInt;
use eyre::Result;
use noodles::bam::Record;
use noodles::sam::alignment::record::cigar::Op;
use noodles::sam::alignment::record::cigar::op::Kind;

use super::cursor::{CigarCursor, ReadCursor, RoiCursor};
use crate::pileup::Pileup;

pub(super) fn process<Cnts: PrimUInt>(
    roi: Interval<usize>,
    pileup: &mut Pileup<Cnts>,
    record: &Record,
    min_phread: u8,
) -> Result<()> {
    debug_assert_eq!(pileup.len(), roi.len());

    // BAM records are 1-based, so we need to subtract 1 to get the 0-based reference position.
    let alignment_start = record
        .alignment_start()
        .transpose()?
        .ok_or_else(|| eyre::eyre!("record does not have an alignment start"))?;
    let mut reference_position = alignment_start.get() - 1;

    // If the reference position is already past the end of the roi, we can skip processing the record.
    // This can happen if the record starts after the roi.
    if reference_position >= roi.end() {
        return Ok(());
    }

    let record_cigar = record.cigar();
    let Some(mut cigar) = CigarCursor::new(record_cigar.iter())? else {
        return Ok(());
    };
    let mut read = ReadCursor::new(record);

    // Fast skip to the roi start if needed
    if reference_position < roi.start() {
        let reached_roi_start =
            skip_reference(&mut cigar, &mut read, roi.start() - reference_position)?;
        if !reached_roi_start {
            return Ok(());
        }
        reference_position = roi.start();
    }

    if reference_position >= roi.end() {
        return Ok(());
    }

    if cigar.is_empty() && !cigar.advance()? {
        return Ok(());
    }

    // We are inside the ROI. Cigar is loaded and ready to be processed.
    debug_assert!(!cigar.is_empty());
    debug_assert!(reference_position >= roi.start() && reference_position < roi.end());
    let mut roi = RoiCursor::new(pileup, reference_position - roi.start());
    process_roi(&mut cigar, &mut read, &mut roi, min_phread)
}

fn process_roi<I, E, Cnts>(
    cigar: &mut CigarCursor<I>,
    read: &mut ReadCursor<'_>,
    roi: &mut RoiCursor<'_, Cnts>,
    min_phread: u8,
) -> Result<()>
where
    I: Iterator<Item = std::result::Result<Op, E>>,
    E: std::error::Error + Send + Sync + 'static,
    Cnts: PrimUInt,
{
    debug_assert!(!cigar.is_empty());
    debug_assert!(!roi.is_exhausted());

    let mut step;
    loop {
        match cigar.op() {
            Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                step = cigar.len().min(roi.remaining());
                roi.aligned(read, step, min_phread);
                read.advance(step);
                roi.advance(step);
            }
            Kind::Deletion => {
                step = cigar.len().min(roi.remaining());
                roi.deletion(step);
                roi.advance(step);
            }
            Kind::Skip => {
                step = cigar.len().min(roi.remaining());
                roi.advance(step);
            }
            Kind::Insertion | Kind::SoftClip => {
                read.advance(cigar.len());
            }
            Kind::HardClip | Kind::Pad => {
                // Do nothing, these do not consume reference or query.
            }
        }

        // The roi is either exhausted and we didn't step fully
        // Or we stepped fully and the cigar is exhausted.
        if roi.is_exhausted() || !cigar.advance()? {
            break;
        }
    }

    Ok(())
}

fn skip_reference<I, E>(
    cigar: &mut CigarCursor<I>,
    read: &mut ReadCursor<'_>,
    mut distance: usize,
) -> Result<bool>
where
    I: Iterator<Item = std::result::Result<Op, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    if distance == 0 {
        return Ok(true);
    }

    let mut step;
    loop {
        match cigar.op() {
            Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                step = distance.min(cigar.len());
                read.advance(step);
            }
            Kind::Deletion | Kind::Skip => {
                step = distance.min(cigar.len());
            }
            Kind::Insertion | Kind::SoftClip => {
                step = 0;
                read.advance(cigar.len());
            }
            Kind::HardClip | Kind::Pad => {
                step = 0;
            }
        }
        distance -= step;
        if distance == 0 {
            cigar.consume(step); // Consume the part of the cigar that we stepped through.
            return Ok(true);
        } else if !cigar.advance()? {
            return Ok(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::num::NonZero;
    use std::ops::Range;

    use biobit_core_rs::loc::Interval;
    use eyre::Result;
    use noodles::bam;
    use noodles::core::Position;
    use noodles::sam;
    use noodles::sam::alignment::record::cigar::Op;
    use noodles::sam::alignment::record::cigar::op::Kind;
    use noodles::sam::alignment::record::{Flags, MappingQuality};
    use noodles::sam::alignment::record_buf::{QualityScores, Sequence};
    use noodles::sam::header::record::value::{Map, map::ReferenceSequence};

    use crate::dna::Observed;
    use crate::pileup::Pileup;

    use super::process;

    const MIN_PHRED: u8 = 20;
    const LEGACY_COORDINATE_SHIFT: i64 = 10;
    const Z: &[Observed] = &[];
    const A: &[Observed] = &[Observed::A];
    const C: &[Observed] = &[Observed::C];
    const G: &[Observed] = &[Observed::G];
    const T: &[Observed] = &[Observed::T];
    const N: &[Observed] = &[Observed::N];
    const D: &[Observed] = &[Observed::Deletion];

    fn pileup(start: usize, end: usize) -> (Interval<usize>, Pileup<u8>) {
        (
            Interval::new(start, end).unwrap(),
            Pileup::zeros(end - start),
        )
    }

    fn record(alignment_start: usize, cigar: Vec<Op>, sequence: &[u8]) -> Result<bam::Record> {
        record_with_qualities(
            alignment_start,
            cigar,
            sequence,
            &vec![MIN_PHRED; sequence.len()],
        )
    }

    fn record_with_qualities(
        alignment_start: usize,
        cigar: Vec<Op>,
        sequence: &[u8],
        quality_scores: &[u8],
    ) -> Result<bam::Record> {
        assert_eq!(sequence.len(), quality_scores.len());
        let header = sam::Header::builder()
            .add_reference_sequence(
                "chr1",
                Map::<ReferenceSequence>::new(const { NonZero::new(1000).unwrap() }),
            )
            .build();
        let record = sam::alignment::RecordBuf::builder()
            .set_name("r0")
            .set_flags(Flags::empty())
            .set_reference_sequence_id(0)
            .set_alignment_start(Position::try_from(alignment_start)?)
            .set_mapping_quality(MappingQuality::try_from(60)?)
            .set_cigar(cigar.into_iter().collect())
            .set_sequence(Sequence::from(sequence))
            .set_quality_scores(QualityScores::from(quality_scores.to_vec()))
            .build();

        let mut writer = bam::io::Writer::from(Vec::new());
        sam::alignment::io::Write::write_alignment_record(&mut writer, &header, &record)?;
        let mut reader = bam::io::Reader::from(Cursor::new(writer.into_inner()));
        let mut record = bam::Record::default();
        assert!(reader.read_record(&mut record)? > 0);
        Ok(record)
    }

    fn high_quality(len: usize) -> Vec<u8> {
        vec![MIN_PHRED; len]
    }

    fn counts(expected: &[&[Observed]], observed: Observed) -> Vec<u8> {
        expected
            .iter()
            .map(|site| {
                site.iter()
                    .filter(|&&site_observed| site_observed == observed)
                    .count() as u8
            })
            .collect()
    }

    fn assert_observed(pileup: &Pileup<u8>, expected: &[&[Observed]]) {
        assert_eq!(pileup.len(), expected.len());
        assert_eq!(pileup.a(), counts(expected, Observed::A));
        assert_eq!(pileup.c(), counts(expected, Observed::C));
        assert_eq!(pileup.g(), counts(expected, Observed::G));
        assert_eq!(pileup.t(), counts(expected, Observed::T));
        assert_eq!(pileup.n(), counts(expected, Observed::N));
        assert_eq!(pileup.deletion(), counts(expected, Observed::Deletion));
    }

    fn run_case(
        roi: Range<i64>,
        pos: i64,
        sequence: &[u8],
        quality_scores: &[u8],
        cigar: Vec<Op>,
        expected: &[&[Observed]],
    ) -> Result<()> {
        assert_eq!((roi.end - roi.start) as usize, expected.len());

        let start = LEGACY_COORDINATE_SHIFT + roi.start;
        let end = LEGACY_COORDINATE_SHIFT + roi.end;
        let alignment_start = usize::try_from(LEGACY_COORDINATE_SHIFT + pos + 1)?;

        let (roi, mut pileup) = pileup(start as usize, end as usize);
        let record = record_with_qualities(alignment_start, cigar, sequence, quality_scores)?;

        process(roi, &mut pileup, &record, MIN_PHRED)?;

        assert_observed(&pileup, expected);
        Ok(())
    }

    #[test]
    fn process_record_counts_overlapping_bases() -> Result<()> {
        let (roi, mut pileup) = pileup(10, 14);
        let record = record(10, vec![Op::new(Kind::Match, 5)], b"TACGT")?;

        process(roi, &mut pileup, &record, MIN_PHRED)?;

        assert_eq!(pileup.a(), &[1, 0, 0, 0]);
        assert_eq!(pileup.c(), &[0, 1, 0, 0]);
        assert_eq!(pileup.g(), &[0, 0, 1, 0]);
        assert_eq!(pileup.t(), &[0, 0, 0, 1]);
        Ok(())
    }

    #[test]
    fn process_record_counts_indels_and_skips() -> Result<()> {
        let (roi, mut pileup) = pileup(10, 17);
        let record = record(
            11,
            vec![
                Op::new(Kind::SoftClip, 1),
                Op::new(Kind::Match, 2),
                Op::new(Kind::Insertion, 2),
                Op::new(Kind::Match, 1),
                Op::new(Kind::Deletion, 2),
                Op::new(Kind::Skip, 1),
                Op::new(Kind::Match, 1),
            ],
            b"XACGGTA",
        )?;

        process(roi, &mut pileup, &record, MIN_PHRED)?;

        assert_eq!(pileup.a(), &[1, 0, 0, 0, 0, 0, 1]);
        assert_eq!(pileup.c(), &[0, 1, 0, 0, 0, 0, 0]);
        assert_eq!(pileup.t(), &[0, 0, 1, 0, 0, 0, 0]);
        assert_eq!(pileup.deletion(), &[0, 0, 0, 1, 1, 0, 0]);
        Ok(())
    }

    #[test]
    fn process_record_filters_low_quality_bases() -> Result<()> {
        let (roi, mut pileup) = pileup(10, 14);
        let record = record_with_qualities(
            11,
            vec![Op::new(Kind::Match, 4)],
            b"ACGT",
            &[MIN_PHRED, MIN_PHRED - 1, MIN_PHRED, MIN_PHRED - 1],
        )?;

        process(roi, &mut pileup, &record, MIN_PHRED)?;

        assert_observed(&pileup, &[A, Z, G, Z]);
        Ok(())
    }

    #[test]
    fn boundary_query_consuming_cases() -> Result<()> {
        for kind in [Kind::Match, Kind::SequenceMismatch, Kind::SequenceMatch] {
            // Full overlap
            run_case(
                0..4,
                0,
                b"ACGT",
                &high_quality(4),
                vec![Op::new(kind, 4)],
                &[A, C, G, T],
            )?;
            // Partial overlap (starts inside)
            run_case(
                0..4,
                1,
                b"AC",
                &high_quality(2),
                vec![Op::new(kind, 2)],
                &[Z, A, C, Z],
            )?;
            // Out of bounds (after)
            run_case(
                0..4,
                4,
                b"ACGT",
                &high_quality(4),
                vec![Op::new(kind, 4)],
                &[Z, Z, Z, Z],
            )?;
            // Partial overlap (ends after)
            run_case(
                0..4,
                2,
                b"ACGT",
                &high_quality(4),
                vec![Op::new(kind, 4)],
                &[Z, Z, A, C],
            )?;
            // Partial overlap (starts before)
            run_case(
                0..4,
                -1,
                b"ACGT",
                &high_quality(4),
                vec![Op::new(kind, 4)],
                &[C, G, T, Z],
            )?;
        }
        Ok(())
    }

    #[test]
    fn boundary_non_counting_reference_and_query_cases() -> Result<()> {
        for kind in [Kind::Skip, Kind::HardClip, Kind::Pad, Kind::SoftClip] {
            let (seq, qual) = if kind == Kind::SoftClip {
                (b"ACGT".as_slice(), high_quality(4))
            } else {
                (b"".as_slice(), vec![])
            };

            let (seq_short, qual_short) = if kind == Kind::SoftClip {
                (b"AC".as_slice(), high_quality(2))
            } else {
                (b"".as_slice(), vec![])
            };

            run_case(0..4, 0, seq, &qual, vec![Op::new(kind, 4)], &[Z, Z, Z, Z])?;
            run_case(
                0..4,
                1,
                seq_short,
                &qual_short,
                vec![Op::new(kind, 2)],
                &[Z, Z, Z, Z],
            )?;
            run_case(0..4, 4, seq, &qual, vec![Op::new(kind, 4)], &[Z, Z, Z, Z])?;
            run_case(0..4, 2, seq, &qual, vec![Op::new(kind, 4)], &[Z, Z, Z, Z])?;
            run_case(0..4, -1, seq, &qual, vec![Op::new(kind, 4)], &[Z, Z, Z, Z])?;
        }
        Ok(())
    }

    #[test]
    fn boundary_deletion_cases() -> Result<()> {
        run_case(
            0..4,
            0,
            b"",
            &[],
            vec![Op::new(Kind::Deletion, 4)],
            &[D, D, D, D],
        )?;
        run_case(
            0..4,
            1,
            b"",
            &[],
            vec![Op::new(Kind::Deletion, 2)],
            &[Z, D, D, Z],
        )?;
        run_case(
            0..4,
            4,
            b"",
            &[],
            vec![Op::new(Kind::Deletion, 4)],
            &[Z, Z, Z, Z],
        )?;
        run_case(
            0..4,
            2,
            b"",
            &[],
            vec![Op::new(Kind::Deletion, 4)],
            &[Z, Z, D, D],
        )?;
        run_case(
            0..4,
            -1,
            b"",
            &[],
            vec![Op::new(Kind::Deletion, 4)],
            &[D, D, D, Z],
        )?;
        Ok(())
    }

    #[test]
    fn boundary_insertion_cases() -> Result<()> {
        run_case(
            0..4,
            0,
            b"ACGT",
            &high_quality(4),
            vec![Op::new(Kind::Insertion, 4)],
            &[Z, Z, Z, Z],
        )?;
        run_case(
            0..4,
            1,
            b"AC",
            &high_quality(2),
            vec![Op::new(Kind::Insertion, 2)],
            &[Z, Z, Z, Z],
        )?;
        run_case(
            0..4,
            4,
            b"ACGT",
            &high_quality(4),
            vec![Op::new(Kind::Insertion, 4)],
            &[Z, Z, Z, Z],
        )?;
        run_case(
            0..4,
            2,
            b"ACGT",
            &high_quality(4),
            vec![Op::new(Kind::Insertion, 4)],
            &[Z, Z, Z, Z],
        )?;
        run_case(
            0..4,
            -1,
            b"ACGT",
            &high_quality(4),
            vec![Op::new(Kind::Insertion, 4)],
            &[Z, Z, Z, Z],
        )?;
        Ok(())
    }

    #[test]
    fn complex_cigar_combinations() -> Result<()> {
        run_case(
            2..5,
            0,
            b"AGC",
            &high_quality(3),
            vec![
                Op::new(Kind::Deletion, 4),
                Op::new(Kind::Match, 1),
                Op::new(Kind::Skip, 3),
                Op::new(Kind::Match, 2),
            ],
            &[D, D, A],
        )?;
        run_case(
            2..5,
            0,
            b"AGC",
            &[MIN_PHRED, MIN_PHRED, MIN_PHRED - 1],
            vec![Op::new(Kind::Match, 3)],
            &[Z, Z, Z],
        )?;
        run_case(
            2..5,
            0,
            b"AGC",
            &high_quality(3),
            vec![
                Op::new(Kind::Match, 1),
                Op::new(Kind::Skip, 2),
                Op::new(Kind::Match, 2),
            ],
            &[Z, G, C],
        )?;
        run_case(
            1..5,
            0,
            b"NNN",
            &high_quality(3),
            vec![
                Op::new(Kind::Match, 1),
                Op::new(Kind::Skip, 2),
                Op::new(Kind::Match, 2),
            ],
            &[Z, Z, N, N],
        )?;
        run_case(
            2..5,
            0,
            b"AGC",
            &high_quality(3),
            vec![
                Op::new(Kind::Deletion, 2),
                Op::new(Kind::Match, 1),
                Op::new(Kind::Skip, 1),
                Op::new(Kind::Match, 1),
                Op::new(Kind::Skip, 1),
                Op::new(Kind::Match, 1),
            ],
            &[A, Z, G],
        )?;
        run_case(
            0..10,
            0,
            b"ACGTNNGTAG",
            &[
                MIN_PHRED,
                MIN_PHRED,
                MIN_PHRED - 1,
                MIN_PHRED,
                MIN_PHRED,
                MIN_PHRED,
                MIN_PHRED,
                MIN_PHRED - 1,
                MIN_PHRED,
                MIN_PHRED,
            ],
            vec![Op::new(Kind::Match, 10)],
            &[A, C, Z, T, N, N, G, Z, A, G],
        )?;
        Ok(())
    }

    #[test]
    fn single_cigar_spanning_both_roi_boundaries() -> Result<()> {
        run_case(
            2..5,
            0,
            b"ACGTACG",
            &high_quality(7),
            vec![Op::new(Kind::Match, 7)],
            &[G, T, A],
        )?;
        run_case(
            2..5,
            0,
            b"",
            &[],
            vec![Op::new(Kind::Deletion, 7)],
            &[D, D, D],
        )?;
        run_case(2..5, 0, b"", &[], vec![Op::new(Kind::Skip, 7)], &[Z, Z, Z])?;
        Ok(())
    }

    #[test]
    fn trailing_insertions_at_interval_end_are_skipped() -> Result<()> {
        run_case(
            0..4,
            0,
            b"ACGTII",
            &high_quality(6),
            vec![Op::new(Kind::Match, 4), Op::new(Kind::Insertion, 2)],
            &[A, C, G, T],
        )?;
        run_case(
            0..4,
            -1,
            b"I",
            &high_quality(1),
            vec![Op::new(Kind::Skip, 5), Op::new(Kind::Insertion, 1)],
            &[Z, Z, Z, Z],
        )?;
        Ok(())
    }

    #[test]
    fn trailing_insertions_after_interval_end_are_ignored() -> Result<()> {
        run_case(
            0..4,
            2,
            b"ACGTII",
            &high_quality(6),
            vec![Op::new(Kind::Match, 4), Op::new(Kind::Insertion, 2)],
            &[Z, Z, A, C],
        )?;
        run_case(
            0..4,
            2,
            b"II",
            &high_quality(2),
            vec![Op::new(Kind::Deletion, 4), Op::new(Kind::Insertion, 2)],
            &[Z, Z, D, D],
        )?;
        Ok(())
    }
}
