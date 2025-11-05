use std::ops::Range;

use dyn_clone::DynClone;

use biobit_core_rs::loc::Interval;
use biobit_io_rs::compression::decode;
use biobit_io_rs::fasta;
use biobit_io_rs::fasta::IndexedReader;
use eyre::Result;
use std::path::PathBuf;

use crate::core::dna::NucCounts;
use crate::core::dna::Nucleotide;

pub trait RefFetcher: Send + DynClone {
    fn fetch(&mut self, contig: &str, range: Range<u64>);
    fn results(&self) -> &[Nucleotide];
}
dyn_clone::clone_trait_object!(RefFetcher);

pub struct FastaRefFetcher {
    reader: Box<dyn fasta::IndexedReaderMutOp + Send>,
    path: PathBuf,
    cache_reader: Vec<u8>,
    cache: Vec<Nucleotide>,
}

impl Clone for FastaRefFetcher {
    fn clone(&self) -> Self {
        Self::new(self.path.clone()).unwrap()
    }
}

impl FastaRefFetcher {
    pub fn new(path: PathBuf) -> Result<Self> {
        let rs = IndexedReader::from_path(&path, &decode::Config::infer_from_path(&path))?;
        Ok(Self {
            reader: rs,
            path,
            cache_reader: Vec::new(),
            cache: Vec::new(),
        })
    }

    #[inline]
    pub fn infer(&self, assembly: Nucleotide, _sequenced: &NucCounts) -> Nucleotide {
        assembly
    }
}

impl RefFetcher for FastaRefFetcher {
    fn fetch(&mut self, seqid: &str, range: Range<u64>) {
        self.reader
            .fetch(
                seqid,
                Interval::new(range.start, range.end).unwrap(),
                &mut self.cache_reader,
            )
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to fetch sequence for region {}:{}-{}",
                    seqid, range.start, range.end
                )
            });

        self.cache.clear();
        let iter = self.cache_reader.iter().map(|x| Nucleotide::from(*x));
        self.cache.extend(iter);

        let expected = (range.end - range.start) as usize;
        if self.cache.len() != expected {
            assert!(self.cache.len() > expected);
            self.cache.truncate(expected);
        }
        debug_assert_eq!(self.cache.len(), expected);
    }

    fn results(&self) -> &[Nucleotide] {
        &self.cache
    }
}
