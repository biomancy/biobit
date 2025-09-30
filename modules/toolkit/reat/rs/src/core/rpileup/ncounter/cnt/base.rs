use crate::core::dna::NucCounts;
use crate::core::read::AlignedRead;
use biobit_core_rs::loc::{Interval, IntervalOp};
use noodles::sam::alignment::record::cigar::op::Kind;
use std::cmp::min;
use std::ops::Range;

#[derive(Clone)]
pub struct BaseNucCounter {
    // Threshold
    min_phread: u8,
    // Caches
    buffer: Vec<NucCounts>,
    matched: Vec<Range<u32>>,
    mapped: u32,
    // Current interval
    seqid: String,
    interval: Interval<u64>,
}

impl BaseNucCounter {
    pub fn new(maxbuf: usize, min_phread: u8) -> Self {
        BaseNucCounter {
            min_phread,
            buffer: Vec::with_capacity(maxbuf),
            matched: Vec::with_capacity(20),
            mapped: 0,
            seqid: String::default(),
            interval: Interval::default(),
        }
    }

    #[inline]
    pub fn seqid(&self) -> &str {
        &self.seqid
    }

    #[inline]
    pub fn interval(&self) -> &Interval<u64> {
        &self.interval
    }

    #[inline]
    pub fn counted(&self) -> &[NucCounts] {
        &self.buffer
    }

    #[inline]
    pub fn mapped(&self) -> u32 {
        self.mapped
    }

    #[inline]
    pub fn reset(&mut self, seqid: String, interval: Interval<u64>) {
        let newlen = interval.len() as usize;
        debug_assert!(newlen > 0);
        self.buffer.clear();
        self.buffer.resize(newlen, NucCounts::zeros());

        self.mapped = 0;
        self.seqid = seqid;
        self.interval = interval;
    }

    pub fn count<R: AlignedRead>(&mut self, read: &R) -> &[Range<u32>] {
        self.matched.clear();

        self.implprocess(read);

        if !self.matched.is_empty() {
            self.mapped += 1;
        }
        &self.matched
    }

    fn implprocess<R: AlignedRead>(&mut self, read: &R) {
        let sequence = read.seq();

        let (mut roipos, mut seqpos) = (read.pos() - self.interval.start() as i64, 0usize);
        let roisize = self.interval.len() as i64;

        let (minseqpos, maxseqpos) = (0, read.len());

        for block in read.cigar().iter() {
            if roipos >= roisize || seqpos >= maxseqpos {
                break;
            }
            let ops = block.len();
            match block.kind() {
                Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                    // fast-end when possible
                    let end = min(roisize as i64, roipos + ops as i64);
                    // fast-forward when possible
                    if roipos < 0 {
                        let skip = min(roipos.abs() as u32, ops as u32);
                        roipos += skip as i64;
                        seqpos += skip as usize;
                    }

                    let mut prevmatched: Option<u32> = None;
                    let start = roipos;
                    for _ in start..end {
                        debug_assert!(roipos < roisize);
                        if seqpos >= minseqpos
                            && seqpos < maxseqpos
                            && read.base_qual(seqpos) >= self.min_phread
                        {
                            debug_assert!(roipos >= 0);
                            // From the SAM specification: No assumptions can be made on the letter cases
                            match sequence[seqpos as usize] {
                                b'A' | b'a' => self.buffer[roipos as usize].A += 1,
                                b'T' | b't' => self.buffer[roipos as usize].T += 1,
                                b'G' | b'g' => self.buffer[roipos as usize].G += 1,
                                b'C' | b'c' => self.buffer[roipos as usize].C += 1,
                                _ => {}
                            }
                            if prevmatched.is_none() {
                                prevmatched = Some(roipos as u32);
                            }
                        } else if let Some(m) = prevmatched {
                            self.matched.push(m..roipos as u32);
                            prevmatched = None;
                        }
                        roipos += 1;
                        seqpos += 1;
                    }
                    if let Some(m) = prevmatched {
                        self.matched.push(m..roipos as u32);
                    }
                }
                Kind::Deletion | Kind::Skip => {
                    roipos += ops as i64;
                }
                Kind::SoftClip | Kind::Insertion => {
                    seqpos += ops as usize;
                }
                Kind::HardClip | Kind::Pad => {}
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use bio_types::strand::ReqStrand;
//     use rust_htslib::bam::record::CigarString;
//     use std::ops::Range;
//
//     use shortcats::*;
//
//     use crate::core::dna::NucCounts;
//     use crate::core::read::MockRead;
//
//     use super::*;
//
//     mod shortcats {
//         #![allow(non_snake_case)]
//
//         use rust_htslib::bam::record::Cigar;
//         use rust_htslib::bam::record::Cigar::*;
//
//         use crate::core::dna::NucCounts;
//
//         pub fn Z() -> NucCounts {
//             NucCounts::zeros()
//         }
//
//         pub fn A() -> NucCounts {
//             NucCounts::A(1)
//         }
//
//         pub fn C() -> NucCounts {
//             NucCounts::C(1)
//         }
//
//         pub fn G() -> NucCounts {
//             NucCounts::G(1)
//         }
//
//         pub fn T() -> NucCounts {
//             NucCounts::T(1)
//         }
//
//         pub fn M(x: u32) -> Cigar {
//             Match(x)
//         }
//
//         pub fn X(x: u32) -> Cigar {
//             Diff(x)
//         }
//
//         pub fn E(x: u32) -> Cigar {
//             Equal(x)
//         }
//
//         pub fn D(x: u32) -> Cigar {
//             Del(x)
//         }
//
//         pub fn N(x: u32) -> Cigar {
//             RefSkip(x)
//         }
//
//         pub fn H(x: u32) -> Cigar {
//             HardClip(x)
//         }
//
//         pub fn P(x: u32) -> Cigar {
//             Pad(x)
//         }
//
//         pub fn S(x: u32) -> Cigar {
//             SoftClip(x)
//         }
//
//         pub fn I(x: u32) -> Cigar {
//             Ins(x)
//         }
//     }
//
//     fn run(
//         roi: Range<u64>,
//         pos: i64,
//         seq: &str,
//         strand: ReqStrand,
//         okbase: Vec<bool>,
//         cigar: Vec<Cigar>,
//         excounts: &[NucCounts],
//         exmatch: &[Range<u32>],
//     ) {
//         let roi = Interval::new("".into(), roi);
//         let roisize = roi.range().end - roi.range().start;
//
//         let mut filter = MockReadsFilter::new();
//         for isok in okbase {
//             filter.expect_is_base_ok().once().return_const(isok);
//         }
//
//         let mut counter = BaseNucCounter::new((roisize + 1) as usize, filter);
//         counter.reset(roi);
//
//         let mut read = MockRead::new();
//         read.expect_pos().return_const(pos);
//         read.expect_len().return_const(seq.len());
//         read.expect_cigar()
//             .return_once(move || CigarString(cigar).into_view(pos));
//         read.expect_strand().return_const(strand);
//         let seq = String::from(seq);
//         read.expect_seq().returning(move || seq.as_bytes().to_vec());
//
//         counter.implprocess(&mut read);
//
//         assert_eq!(counter.buffer, excounts);
//         assert_eq!(counter.matched, exmatch);
//     }
//
//     #[test]
//     #[allow(non_snake_case)]
//     fn implprocess() {
//         // Query consuming operations only
//         for op in [M, X, E] {
//             // complete overlap with the region
//             run(
//                 0..4,
//                 0,
//                 "ACGT",
//                 ReqStrand::Forward,
//                 vec![true, true, true, true],
//                 vec![op(4)],
//                 &[A(), C(), G(), T()],
//                 &[0..4],
//             );
//             // inside region
//             run(
//                 0..4,
//                 1,
//                 "AC",
//                 ReqStrand::Forward,
//                 vec![true, true],
//                 vec![op(2)],
//                 &[Z(), A(), C(), Z()],
//                 &[1..3],
//             );
//             // completely out of the region
//             run(
//                 0..4,
//                 4,
//                 "ACGT",
//                 ReqStrand::Reverse,
//                 vec![],
//                 vec![op(4)],
//                 &[Z(), Z(), Z(), Z()],
//                 &[],
//             );
//             // end out of the region
//             run(
//                 0..4,
//                 2,
//                 "ACGT",
//                 ReqStrand::Forward,
//                 vec![true, true],
//                 vec![op(4)],
//                 &[Z(), Z(), A(), C()],
//                 &[2..4],
//             );
//             // start out of the region
//             run(
//                 0..4,
//                 -1,
//                 "ACGT",
//                 ReqStrand::Reverse,
//                 vec![true, true, true],
//                 vec![op(4)],
//                 &[C(), G(), T(), Z()],
//                 &[0..3],
//             );
//         }
//
//         // No-ops + reference/query consuming operations only
//         let empty = vec![Z()].repeat(4);
//         for op in [D, N, H, P, S, I] {
//             // complete overlap with the region
//             run(
//                 0..4,
//                 0,
//                 "ACGT",
//                 ReqStrand::Forward,
//                 vec![],
//                 vec![op(4)],
//                 &empty,
//                 &[],
//             );
//             // inside region
//             run(
//                 0..4,
//                 1,
//                 "AC",
//                 ReqStrand::Reverse,
//                 vec![],
//                 vec![op(2)],
//                 &empty,
//                 &[],
//             );
//             // completely out of the region
//             run(
//                 0..4,
//                 4,
//                 "ACGT",
//                 ReqStrand::Forward,
//                 vec![],
//                 vec![op(4)],
//                 &empty,
//                 &[],
//             );
//             // end out of the region
//             run(
//                 0..4,
//                 2,
//                 "ACGT",
//                 ReqStrand::Reverse,
//                 vec![],
//                 vec![op(4)],
//                 &empty,
//                 &[],
//             );
//             // start out of the region
//             run(
//                 0..4,
//                 -1,
//                 "ACGT",
//                 ReqStrand::Forward,
//                 vec![],
//                 vec![op(4)],
//                 &empty,
//                 &[],
//             );
//         }
//
//         // Complex queries
//         run(
//             2..5,
//             0,
//             "AGC",
//             ReqStrand::Forward,
//             vec![true],
//             vec![D(4), M(1), N(3), M(2)],
//             &[Z(), Z(), A()],
//             &[2..3],
//         );
//         run(
//             2..5,
//             0,
//             "AGC",
//             ReqStrand::Reverse,
//             vec![false],
//             vec![M(3)],
//             &[Z(), Z(), Z()],
//             &[],
//         );
//         run(
//             2..5,
//             0,
//             "AGC",
//             ReqStrand::Reverse,
//             vec![true, true],
//             vec![M(1), N(2), M(2)],
//             &[Z(), G(), C()],
//             &[1..3],
//         );
//         run(
//             1..5,
//             0,
//             "NNN",
//             ReqStrand::Forward,
//             vec![true, true],
//             vec![M(1), N(2), M(2)],
//             &[Z(), Z(), Z(), Z()],
//             &[2..4],
//         );
//
//         run(
//             2..5,
//             0,
//             "AGC",
//             ReqStrand::Forward,
//             vec![true, true],
//             vec![D(2), M(1), N(1), M(1), N(1), M(1)],
//             &[A(), Z(), G()],
//             &[0..1, 2..3],
//         );
//         run(
//             0..10,
//             0,
//             "ACGTNNGTAG",
//             ReqStrand::Reverse,
//             vec![true, true, false, true, true, true, true, false, true, true],
//             vec![M(10)],
//             &[A(), C(), Z(), T(), Z(), Z(), G(), Z(), A(), G()],
//             &[0..2, 3..7, 8..10],
//         );
//     }
//
//     #[test]
//     fn is_record_ok() {
//         let contig = "".to_string();
//         let wrong_contig = "!".to_string();
//         for (isok, ctg, result) in [
//             (true, &contig, true),
//             (false, &contig, false),
//             (true, &wrong_contig, false),
//             (false, &wrong_contig, false),
//         ] {
//             let mut filter = MockReadsFilter::new();
//             filter.expect_is_read_ok().once().return_const(isok);
//             let mut dummy = BaseNucCounter::new(1, filter);
//             dummy.reset(Interval::new(contig.clone(), 0..1));
//
//             let mut read = MockRead::new();
//             read.expect_contig().return_const(ctg.clone());
//             assert_eq!(dummy.is_record_ok(&mut read), result)
//         }
//     }
// }
