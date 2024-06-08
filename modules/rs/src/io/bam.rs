use std::fmt::{Display, Formatter, Pointer};
use std::fs::File;
use std::path::PathBuf;

use noodles::{bam, sam};
use noodles::sam::alignment::record::Flags;

use super::alignment_segments_source::{AlignmentSegments, AlignmentSegmentsSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessedReads {
    // Single-end reads
    pub single: u64,
    // Pair-end reads & related flags
    pub paired: u64,
    pub read_1: u64,
    pub read_2: u64,
    pub proper_pair: u64,
    pub mate_mapped: u64,
    pub singleton: u64,
    // Alignment types
    pub unmapped: u64,
    pub primary: u64,
    pub secondary: u64,
    pub supplementary: u64,
    pub duplicate: u64,
    pub fails_quality_checks: u64,
    // Stranding information
    pub reverse_strand: u64,
    pub mate_reverse_strand: u64,
    // Total number of processed reads
    pub total: u64,
}

impl ProcessedReads {
    pub fn new() -> Self {
        Self {
            single: 0,
            paired: 0,
            read_1: 0,
            read_2: 0,
            proper_pair: 0,
            mate_mapped: 0,
            singleton: 0,
            unmapped: 0,
            primary: 0,
            secondary: 0,
            supplementary: 0,
            duplicate: 0,
            fails_quality_checks: 0,
            reverse_strand: 0,
            mate_reverse_strand: 0,
            total: 0,
        }
    }

    pub fn count(&mut self, flags: &Flags) {
        if flags.is_unmapped() {
            self.unmapped += 1;
        } else if flags.is_secondary() {
            self.secondary += 1;
        } else if flags.is_supplementary() {
            self.supplementary += 1;
        } else {
            self.primary += 1;
        }

        if flags.is_qc_fail() {
            self.fails_quality_checks += 1;
        }
        if flags.is_duplicate() {
            self.duplicate += 1;
        }
        if flags.is_reverse_complemented() {
            self.reverse_strand += 1;
        }

        match flags.is_segmented() {
            true => {
                self.paired += 1;
                if flags.is_first_segment() {
                    self.read_1 += 1;
                } else if flags.is_last_segment() {
                    self.read_2 += 1;
                }
                if flags.is_properly_segmented() {
                    self.proper_pair += 1;
                }
                if !flags.is_unmapped() && flags.is_mate_unmapped() {
                    self.singleton += 1;
                } else if flags.is_mate_unmapped() {
                    self.mate_mapped += 1;
                }
                if flags.is_properly_segmented() {
                    self.proper_pair += 1;
                }
                if flags.is_mate_reverse_complemented() {
                    self.mate_reverse_strand += 1;
                }
            }
            false => {
                self.single += 1;
            }
        }

        self.total += 1;
    }
}

impl Default for ProcessedReads { fn default() -> Self { Self::new() } }

impl Display for ProcessedReads {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Single-end reads: {}\n", self.single)?;
        write!(f, "Pair-end reads: {}\n", self.paired)?;
        write!(f, "Read 1: {}\n", self.read_1)?;
        write!(f, "Read 2: {}\n", self.read_2)?;
        write!(f, "Proper pair: {}\n", self.proper_pair)?;
        write!(f, "Mate mapped: {}\n", self.mate_mapped)?;
        write!(f, "Singleton: {}\n", self.singleton)?;
        write!(f, "Unmapped: {}\n", self.unmapped)?;
        write!(f, "Primary: {}\n", self.primary)?;
        write!(f, "Secondary: {}\n", self.secondary)?;
        write!(f, "Supplementary: {}\n", self.supplementary)?;
        write!(f, "Duplicate: {}\n", self.duplicate)?;
        write!(f, "Fails quality checks: {}\n", self.fails_quality_checks)?;
        write!(f, "Reverse strand: {}\n", self.reverse_strand)?;
        write!(f, "Mate reverse strand: {}\n", self.mate_reverse_strand)?;
        write!(f, "Total: {}\n", self.total)
    }
}


pub struct BasicReader {
    filename: PathBuf,
    inflags: u16,
    exflags: u16,
    minmapq: u8,
    consumed: ProcessedReads,
    discarded: ProcessedReads,
}

impl BasicReader {
    pub fn new(filename: &std::path::Path, inflags: u16, exflags: u16, minmapq: u8) -> Self {
        Self {
            filename: filename.to_path_buf(),
            inflags,
            exflags,
            minmapq,
            consumed: ProcessedReads::new(),
            discarded: ProcessedReads::new(),
        }
    }
}

impl AlignmentSegmentsSource for BasicReader {
    type Idx = u64;
    type Stats = ProcessedReads;
    type Iter<'a> = BasicReaderIterator<'a, File>;

    fn fetch(&mut self, contig: &str, start: Self::Idx, end: Self::Idx) -> Self::Iter<'_> {
        let mut reader = bam::io::indexed_reader::Builder::default()
            .build_from_path(&self.filename)
            .expect("Failed to build the bam file");
        let header = reader.read_header().expect("Failed to read the header");

        BasicReaderIterator {
            bam: reader,
            header,
            record: Default::default(),
            parent: &mut self,
        }
    }

    fn stats(&self) -> Self::Stats {
        todo!()
    }
}


pub struct BasicReaderIterator<'a, T> {
    pub bam: bam::io::IndexedReader<bam::io::Reader<T>>,
    pub header: sam::Header,
    pub record: bam::Record,
    pub parent: &'a mut BasicReader,
}

impl<'a, T> Iterator for BasicReaderIterator<'a, T> {
    type Item = AlignmentSegments<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
    
}


// What do I want to do?
// How to organize the iterator?
// How to organize stats backpropagation?
// How to organize the cache?
// How to organize the bundler?
// How to organize everything else?
// I want to do what?...
// How to do it?...
// 1. Ideally - I want to hold the iterator until the reading is complete and then close the file.


pub struct PEReadsCache {
    lmates: Vec<bam::Record>,
    rmates: Vec<bam::Record>,
}


// class Reader:
//     def __init__(self, filename: Path, inflags: int, exflags: int, minmapq: int, *, statistics: bool = False):
//         self.filename: Path = filename
//         self.sf: AlignmentFile | None = None
//         self.iterator: Iterable[AlignedSegment] | None = None
//         self.inflags: int = inflags
//         self.exflags: int = exflags
//         self.minmapq: int = minmapq
//
//         self.statistics: bool = statistics
//         self.consumed: ConsumedReads | None = None if not statistics else ConsumedReads()
//         self.discarded: ConsumedReads | None = None if not statistics else ConsumedReads()
//
//     def fetch(self, contig: str, start: int | None = None, end: int | None = None) -> Self:
//         if self.sf is None:
//             self.sf = AlignmentFile(self.filename.as_posix(), "rb")
//         self.iterator = self.sf.fetch(contig, start, end)
//         return self
//
//     def _is_read_ok(self, segment: AlignedSegment) -> bool:
//         return (
//                 segment.flag & self.inflags == self.inflags
//                 and segment.flag & self.exflags == 0
//                 and segment.mapping_quality >= self.minmapq
//         )
//
//     def _iter_with_statistics(self) -> Iterator[AlignedSegment]:
//         assert self.consumed is not None and self.discarded is not None
//         assert self.iterator is not None
//
//         for segment in self.iterator:
//             if self._is_read_ok(segment):
//                 self.consumed.count(segment)
//                 yield segment
//             else:
//                 self.discarded.count(segment)
//
//         self.iterator = None
//
//     def _iter_without_statistics(self) -> Iterator[AlignedSegment]:
//         assert self.iterator is not None
//         for segment in self.iterator:
//             if self._is_read_ok(segment):
//                 yield segment
//
//         self.iterator = None
//
//     def __iter__(self) -> Iterator[AlignedSegment]:
//         if self.sf is None:
//             self.sf = AlignmentFile(self.filename.as_posix(), "rb")
//             self.iterator = self.sf
//
//         if self.iterator is None:
//             self.iterator = self.sf
//
//         if self.statistics:
//             return self._iter_with_statistics()
//         else:
//             return self._iter_without_statistics()
//
//     def __deepcopy__(self, _):
//         return Reader(self.filename, self.inflags, self.exflags, self.minmapq, statistics=self.statistics)
//
//     def __getstate__(self):
//         return self.filename, self.inflags, self.exflags, self.minmapq, self.statistics
//
//     def __setstate__(self, state):
//         self.filename, self.inflags, self.exflags, self.minmapq, self.statistics = state
//         self.sf = None
//         self.consumed = None if not self.statistics else ConsumedReads()
//         self.discarded = None if not self.statistics else ConsumedReads()
//
//
// @define(slots=True)
// class _PEReadsCache:
//     lmates: list[AlignedSegment] = field(factory=list)
//     rmates: list[AlignedSegment] = field(factory=list)
//
//     def bundle(self) -> Iterator[tuple[AlignedSegment, AlignedSegment]]:
//         missed_lmates = []
//         for lmate in self.lmates:
//             mate_found = False
//             for ind, rmate in enumerate(self.rmates):
//                 if (
//                         lmate.next_reference_id == rmate.reference_id
//                         and lmate.next_reference_start == rmate.reference_start
//                         and lmate.mate_is_reverse == rmate.is_reverse
//                         and lmate.mate_is_unmapped == rmate.is_unmapped
//                         and rmate.next_reference_id == lmate.reference_id
//                         and rmate.next_reference_start == lmate.reference_start
//                         and rmate.mate_is_reverse == lmate.is_reverse
//                         and rmate.mate_is_unmapped == lmate.is_unmapped
//                 ):
//                     mate_found = True
//                     break
//
//             if not mate_found:
//                 missed_lmates.append(lmate)
//             else:
//                 rmate = self.rmates.pop(ind)
//                 yield lmate, rmate
//
//         self.lmates = missed_lmates
//
//     def is_empty(self) -> bool:
//         return len(self.lmates) == 0 and len(self.rmates) == 0
//
//
// class PEReadsBundler:
//     def __init__(self, reader: Reader, step: int = 100_000):
//         self.reader: Reader = reader
//         self.cache: dict[str, _PEReadsCache] = defaultdict(_PEReadsCache)
//
//         self.step: int = step
//         self.unpaired = 0
//
//     def fetch(self, contig: str, start: int | None = None, end: int | None = None) -> Self:
//         self.reader.fetch(contig, start, end)
//         return self
//
//     def __iter__(self) -> Iterator[tuple[AlignedSegment, AlignedSegment]]:
//         for ind, segment in enumerate(self.reader):
//             qname = segment.query_name
//             assert qname is not None, "Query name must be present in the BAM file"
//
//             cached = self.cache[qname]
//             if segment.is_read1:
//                 cached.lmates.append(segment)
//             else:
//                 cached.rmates.append(segment)
//
//             if ind % self.step == 0 and ind > 0:
//                 for k, v in list(self.cache.items()):
//                     yield from v.bundle()
//                     if v.is_empty():
//                         self.cache.pop(k)
//
//         # Clean up the cache
//         for k, v in list(self.cache.items()):
//             yield from v.bundle()
//             self.unpaired += len(v.lmates) + len(v.rmates)
//         self.cache.clear()
//
//     def summarize(self, prefix: str = "") -> str:
//         return f"{prefix}Unpaired reads left: {self.unpaired}\n"