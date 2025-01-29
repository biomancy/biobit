use std::hash::{Hash, Hasher};
use std::io;

use ahash::HashSet;
use derive_getters::Dissolve;
use noodles::bam::Record;
use noodles::sam::alignment::record::data::field::{Tag, Value};

#[derive(Debug, Clone, Dissolve)]
struct CachedRecord {
    record: Record,
    hit_index: i8,
}

impl Hash for CachedRecord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hit_index.hash(state);
        match self.record.name() {
            Some(name) => name.hash(state),
            None => 0.hash(state),
        }
    }
}

impl PartialEq for CachedRecord {
    fn eq(&self, other: &Self) -> bool {
        self.hit_index == other.hit_index && self.record.name() == other.record.name()
    }
}

impl Eq for CachedRecord {}

impl TryInto<CachedRecord> for Record {
    type Error = io::Error;

    fn try_into(self) -> Result<CachedRecord, Self::Error> {
        let hit_index = {
            let data = self.data();
            let tag = data.get(&Tag::HIT_INDEX);
            let hit_index = match tag {
                Some(Ok(tag)) => tag,
                Some(Err(e)) => return Err(e),
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "No HIT_INDEX tag",
                    ))
                }
            };
            match hit_index {
                Value::Int8(tag) => tag,
                Value::UInt8(tag) => tag as i8,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "HIT_INDEX tag must be an int8 or uint8",
                    ))
                }
            }
        };

        Ok(CachedRecord {
            record: self,
            hit_index,
        })
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
        // let rname = record.name().map(|x| x.to_owned());
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

        // Otherwise, insert the record into the cache
        let inserted = if is_lmate {
            self.lmate.insert(record)
        } else {
            self.rmate.insert(record)
        };

        // If the record is already in the cache, log an error
        if !inserted {
            log::error!("Double insert in the cache detected, check that read names and HIT_INDEX tags are unique");
            // println!("{:?}", rname);
        }
        debug_assert!(inserted);

        Ok(None)
    }
}
