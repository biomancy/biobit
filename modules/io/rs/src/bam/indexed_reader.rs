use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::path::Path;

use derive_getters::Dissolve;
use noodles::{bam, bgzf, csi::BinningIndex};

#[derive(Dissolve)]
pub struct IndexedReader<R> {
    pub inner: bam::io::Reader<R>,
    pub index: Box<dyn BinningIndex + Send + Sync>,
}

impl IndexedReader<bgzf::Reader<File>> {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut index = OsString::from(path.as_ref());
        index.push(".");
        index.push("bai");

        let index = bam::bai::read(index)?;
        let file = File::open(path)?;

        Ok(Self {
            inner: bam::io::Reader::new(file),
            index: Box::new(index),
        })
    }
}
