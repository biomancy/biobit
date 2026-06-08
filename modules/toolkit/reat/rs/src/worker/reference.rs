use biobit_core_rs::loc::IntervalOp;
use biobit_core_rs::num::PrimUInt;
use biobit_io_rs::fasta::IndexedReaderMutOp;
use eyre::{Result, ensure};

use crate::dna;

pub struct RefReader {
    reader: Box<dyn IndexedReaderMutOp + Send + Sync>,
    raw: Vec<u8>,
    reference: Vec<dna::Reference>,
}

impl RefReader {
    pub fn with_capacity(
        reader: Box<dyn IndexedReaderMutOp + Send + Sync>,
        capacity: usize,
    ) -> Self {
        Self {
            reader,
            raw: Vec::with_capacity(capacity),
            reference: Vec::with_capacity(capacity),
        }
    }

    pub fn fetch<Iv, Idx>(&mut self, seqid: &str, interval: Iv) -> Result<()>
    where
        Iv: IntervalOp<Idx = Idx>,
        Idx: PrimUInt,
    {
        let interval = interval.as_interval();
        let fetched = interval.cast::<u64>().ok_or_else(|| {
            eyre::eyre!("task envelope does not fit into reference coordinate type")
        })?;
        let expected_len = interval
            .len()
            .to_usize()
            .ok_or_else(|| eyre::eyre!("task envelope length does not fit into usize"))?;

        self.reader.fetch(seqid, fetched, &mut self.raw)?;
        ensure!(
            self.raw.len() == expected_len,
            "fetched reference length does not match task envelope length"
        );

        self.reference.clear();
        for byte in &self.raw {
            let base = match dna::Reference::try_from(*byte) {
                Ok(base) => base,
                Err(_) => {
                    eyre::bail!(
                        "invalid reference base '{}' (byte value {})",
                        *byte as char,
                        *byte
                    );
                }
            };
            self.reference.push(base);
        }

        Ok(())
    }

    pub fn reference(&self) -> &[dna::Reference] {
        &self.reference
    }
}
