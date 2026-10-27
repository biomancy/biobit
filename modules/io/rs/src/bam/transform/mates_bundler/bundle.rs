use std::hash::{Hash, Hasher};
use std::io;

use ahash::HashSet;
use derive_getters::Dissolve;
use noodles::bam::Record;
use noodles::sam::alignment::record::data::field::Tag;

#[derive(Debug, Clone, Dissolve)]
struct CachedRecord {
    record: Record,
    hit_index: u32,
}

impl Hash for CachedRecord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hit_index.hash(state);
        self.record.name().hash(state);
    }
}

impl PartialEq for CachedRecord {
    fn eq(&self, other: &Self) -> bool {
        self.hit_index == other.hit_index && self.record.name() == other.record.name()
    }
}

impl Eq for CachedRecord {}

impl TryFrom<Record> for CachedRecord {
    type Error = io::Error;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        let flags = record.flags();
        if record.name().is_none()
            || !flags.is_segmented()
            || flags.is_first_segment() == flags.is_last_segment()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Mate bundling requires a read name and exactly one of the first/last segment flags",
            ));
        }
        let hit_index = record
            .data()
            .get(&Tag::HIT_INDEX)
            .transpose()?
            .and_then(|value| value.as_int())
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "BAM HI hit index must be present and be a non-negative integer",
                )
            })?;

        Ok(Self { record, hit_index })
    }
}

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

        // Retain both records on collision without adding a lookup to normal inserts.
        let cache = if is_lmate {
            &mut self.lmate
        } else {
            &mut self.rmate
        };
        if let Some(previous) = cache.replace(record) {
            let incoming = cache
                .get(&previous)
                .expect("a newly inserted mate disappeared");
            let error = io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Duplicate BAM mate: QNAME={:?}, HI={}, mate={}; identical={}. \
                     Existing (flags, POS, PNEXT, RG): {:?}. \
                     Incoming (flags, POS, PNEXT, RG): {:?}",
                    previous.record.name(),
                    previous.hit_index,
                    if is_lmate { "first" } else { "last" },
                    previous.record == incoming.record,
                    (
                        u16::from(previous.record.flags()),
                        previous.record.alignment_start(),
                        previous.record.mate_alignment_start(),
                        previous.record.data().get(&Tag::READ_GROUP),
                    ),
                    (
                        u16::from(incoming.record.flags()),
                        incoming.record.alignment_start(),
                        incoming.record.mate_alignment_start(),
                        incoming.record.data().get(&Tag::READ_GROUP),
                    ),
                ),
            );
            // Preserve the original cached record if the caller inspects or reuses the bundler.
            let _ = cache.replace(previous);
            return Err(error);
        }

        Ok(None)
    }
}
