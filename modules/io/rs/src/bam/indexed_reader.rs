use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Arc;

use derive_getters::Dissolve;
use noodles::{bam, bgzf, csi::BinningIndex};

#[derive(Dissolve)]
pub struct IndexedReader<R> {
    pub inner: bam::io::Reader<R>,
    pub index: Arc<dyn BinningIndex + Send + Sync>,
}

impl IndexedReader<bgzf::io::Reader<File>> {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let index = bam::fs::read_associated_index(path.as_ref())?;
        let file = File::open(path)?;

        Ok(Self {
            inner: bam::io::Reader::new(file),
            index: Arc::new(index),
        })
    }
}
