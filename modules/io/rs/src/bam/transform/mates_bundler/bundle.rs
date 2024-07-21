use std::hash::{Hash, Hasher};
use std::io;

use ahash::HashSet;
use derive_getters::Dissolve;
use noodles::bam::Record;
use noodles::core::Position;

#[derive(Debug, Clone, Dissolve)]
struct CachedRecord {
    record: Record,
    // Self
    is_reverse_complemented: bool,
    is_unmapped: bool,
    reference_sequence_id: usize,
    start: Position,
    // Mate
    is_mate_reverse_complemented: bool,
    is_mate_unmapped: bool,
    mate_reference_sequence_id: usize,
    mate_start: Position,
}

impl CachedRecord {
    fn flip(mut self) -> Self {
        std::mem::swap(
            &mut self.is_reverse_complemented,
            &mut self.is_mate_reverse_complemented,
        );
        std::mem::swap(&mut self.is_unmapped, &mut self.is_mate_unmapped);
        std::mem::swap(
            &mut self.reference_sequence_id,
            &mut self.mate_reference_sequence_id,
        );
        std::mem::swap(&mut self.start, &mut self.mate_start);
        self
    }
}

impl Hash for CachedRecord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_reverse_complemented.hash(state);
        self.is_unmapped.hash(state);
        self.reference_sequence_id.hash(state);
        self.start.hash(state);
        self.is_mate_reverse_complemented.hash(state);
        self.is_mate_unmapped.hash(state);
        self.mate_reference_sequence_id.hash(state);
        self.mate_start.hash(state);
        match self.record.name() {
            Some(name) => name.as_bytes().hash(state),
            None => 0.hash(state),
        }
    }
}

impl PartialEq for CachedRecord {
    fn eq(&self, other: &Self) -> bool {
        self.is_reverse_complemented == other.is_reverse_complemented
            && self.is_unmapped == other.is_unmapped
            && self.reference_sequence_id == other.reference_sequence_id
            && self.start == other.start
            && self.is_mate_reverse_complemented == other.is_mate_reverse_complemented
            && self.is_mate_unmapped == other.is_mate_unmapped
            && self.mate_reference_sequence_id == other.mate_reference_sequence_id
            && self.mate_start == other.mate_start
            && self.record.name() == other.record.name()
    }
}

impl Eq for CachedRecord {}

impl TryInto<CachedRecord> for Record {
    type Error = io::Error;

    fn try_into(self) -> Result<CachedRecord, Self::Error> {
        let start = self.alignment_start().transpose()?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Alignment start must be present in the BAM file",
            )
        })?;
        let reference_sequence_id = self.reference_sequence_id().transpose()?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Reference sequence ID must be present in the BAM file",
            )
        })?;
        let mate_start = self.mate_alignment_start().transpose()?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Mate alignment start must be present in the BAM file",
            )
        })?;
        let mate_reference_sequence_id = self
            .mate_reference_sequence_id()
            .transpose()?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Mate reference sequence ID must be present in the BAM file",
                )
            })?;
        let flags = self.flags();
        Ok(CachedRecord {
            record: self,
            is_reverse_complemented: flags.is_reverse_complemented(),
            is_unmapped: flags.is_unmapped(),
            reference_sequence_id,
            start,
            is_mate_reverse_complemented: flags.is_mate_reverse_complemented(),
            is_mate_unmapped: flags.is_mate_unmapped(),
            mate_reference_sequence_id,
            mate_start,
        })
    }
}

// I need two "records" for the map. The first one will hold left mates and the second one will hold right mates.
// Each field is a map for candidate lmates or rmates.

// The map is 2 level deep, The first level is (reference_sequence_id, reverse_complemented). The second level is (start, name).
#[derive(Debug, Clone, Default, Dissolve)]
pub struct Bundler {
    lmate: HashSet<CachedRecord>,
    rmate: HashSet<CachedRecord>,
}

impl Bundler {
    pub fn clear(&mut self) {
        self.lmate.clear();
        self.rmate.clear();
    }

    pub fn push(&mut self, record: Record) -> io::Result<Option<(Record, Record)>> {
        let is_lmate = record.flags().is_first_segment();

        // Try to look up the mate in the cache
        let record: CachedRecord = record.try_into()?;
        let record = record.flip();

        let entry = if is_lmate {
            self.rmate.take(&record)
        } else {
            self.lmate.take(&record)
        };

        // If the mate is found, return the pair
        if let Some(mate) = entry {
            return if is_lmate {
                Ok(Some((record.record, mate.record)))
            } else {
                Ok(Some((mate.record, record.record)))
            };
        }

        // Otherwise, flip back to self and insert
        let record = record.flip();
        let inserted = if is_lmate {
            self.lmate.insert(record)
        } else {
            self.rmate.insert(record)
        };
        debug_assert!(inserted);

        Ok(None)
    }
}

// #[derive(Debug, Clone, Default, Dissolve)]
// pub struct Bundle {
//     lmate: Vec<BundledRecord>,
//     rmate: Vec<BundledRecord>,
// }
//
// #[derive(Debug, Clone, PartialEq, Dissolve)]
// struct BundledRecord {
//     record: bam::Record,
//     // Self
//     start: Position,
//     reference_sequence_id: usize,
//     flags: Flags,
//     // Mate
//     mate_start: Position,
//     mate_reference_sequence_id: usize,
// }
//
// impl TryInto<BundledRecord> for bam::Record {
//     type Error = io::Error;
//
//     fn try_into(self) -> Result<BundledRecord, Self::Error> {
//         let start = self.alignment_start().transpose()?.ok_or_else(|| {
//             io::Error::new(
//                 io::ErrorKind::InvalidData,
//                 "Alignment start must be present in the BAM file",
//             )
//         })?;
//         let reference_sequence_id = self.reference_sequence_id().transpose()?.ok_or_else(|| {
//             io::Error::new(
//                 io::ErrorKind::InvalidData,
//                 "Reference sequence ID must be present in the BAM file",
//             )
//         })?;
//         let mate_start = self.mate_alignment_start().transpose()?.ok_or_else(|| {
//             io::Error::new(
//                 io::ErrorKind::InvalidData,
//                 "Mate alignment start must be present in the BAM file",
//             )
//         })?;
//         let mate_reference_sequence_id = self
//             .mate_reference_sequence_id()
//             .transpose()?
//             .ok_or_else(|| {
//                 io::Error::new(
//                     io::ErrorKind::InvalidData,
//                     "Mate reference sequence ID must be present in the BAM file",
//                 )
//             })?;
//         let flags = self.flags();
//
//         Ok(BundledRecord {
//             record: self,
//             start,
//             flags,
//             reference_sequence_id,
//             mate_start,
//             mate_reference_sequence_id,
//         })
//     }
// }
//
// pub enum BundlingResult {
//     Empty,
//     Solved(usize),
//     Bundled(usize),
// }
//
// impl Bundle {
//     pub fn clear(&mut self) {
//         self.lmate.clear();
//         self.rmate.clear();
//     }
//
//     pub fn push(&mut self, record: bam::Record) -> io::Result<()> {
//         let record: BundledRecord = record.try_into()?;
//
//         if record.flags.is_first_segment() {
//             self.lmate.push(record);
//         } else {
//             self.rmate.push(record);
//         }
//
//         Ok(())
//     }
//
//     pub fn is_empty(&self) -> bool {
//         self.lmate.is_empty() && self.rmate.is_empty()
//     }
//
//     pub fn try_bundle(&mut self, writeto: &mut Vec<(bam::Record, bam::Record)>) -> BundlingResult {
//         if self.lmate.is_empty() && self.rmate.is_empty() {
//             return BundlingResult::Empty;
//         } else if self.lmate.is_empty() || self.rmate.is_empty() {
//             return BundlingResult::Bundled(0);
//         }
//
//         let mut lmate = 0;
//         let mut bunled = 0;
//
//         while lmate < self.lmate.len() {
//             let mut paired = false;
//
//             for rmate in 0..self.rmate.len() {
//                 let (left, right) = (&self.lmate[lmate], &self.rmate[rmate]);
//                 if left.mate_reference_sequence_id == right.reference_sequence_id
//                     && left.mate_start == right.start
//                     && left.flags.is_mate_reverse_complemented()
//                         == right.flags.is_reverse_complemented()
//                     && left.flags.is_mate_unmapped() == right.flags.is_unmapped()
//                     && right.mate_reference_sequence_id == left.reference_sequence_id
//                     && right.mate_start == left.start
//                     && right.flags.is_mate_reverse_complemented()
//                         == left.flags.is_reverse_complemented()
//                     && right.flags.is_mate_unmapped() == left.flags.is_unmapped()
//                 {
//                     writeto.push((
//                         self.lmate.remove(lmate).record,
//                         self.rmate.remove(rmate).record,
//                     ));
//                     paired = true;
//                     break;
//                 }
//             }
//
//             if !paired {
//                 lmate += 1;
//             } else {
//                 bunled += 1;
//             }
//         }
//
//         if self.lmate.is_empty() && self.rmate.is_empty() {
//             BundlingResult::Solved(bunled)
//         } else {
//             BundlingResult::Bundled(bunled)
//         }
//     }
// }
