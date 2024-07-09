use std::collections::HashMap;
use std::io;

use ::higher_kinded_types::prelude::*;
use noodles::bam::Record;

use biobit_core_rs::LendingIterator;


#[derive(Debug, Clone, PartialEq, Default)]
struct Bundle {
    lmate: Vec<Record>,
    rmate: Vec<Record>,
}

impl Bundle {
    fn new() -> Self {
        Self {
            lmate: Vec::new(),
            rmate: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.lmate.is_empty() && self.rmate.is_empty()
    }

    fn push(&mut self, record: Record) {
        if record.flags().is_first_segment() {
            self.lmate.push(record);
        } else {
            self.rmate.push(record);
        }
    }

    fn try_bundle(&mut self, writeto: &mut Vec<(Record, Record)>) -> io::Result<usize> {
        let mut lmate = 0;
        let mut bunled = 0;

        while lmate < self.lmate.len() {
            let mut paired = false;

            for rmate in 0..self.rmate.len() {
                let (left, right) = (&self.lmate[lmate], &self.rmate[rmate]);
                if (left.mate_reference_sequence_id().transpose()?
                    == right.reference_sequence_id().transpose()?
                    && left.mate_alignment_start().transpose()?
                        == right.alignment_start().transpose()?
                    && left.flags().is_mate_reverse_complemented()
                        == right.flags().is_reverse_complemented()
                    && left.flags().is_mate_unmapped() == right.flags().is_unmapped())
                    && (right.mate_reference_sequence_id().transpose()?
                        == left.reference_sequence_id().transpose()?
                        && right.mate_alignment_start().transpose()?
                            == left.alignment_start().transpose()?
                        && right.flags().is_mate_reverse_complemented()
                            == left.flags().is_reverse_complemented()
                        && right.flags().is_mate_unmapped() == left.flags().is_unmapped())
                {
                    writeto.push((self.lmate.remove(lmate), self.rmate.remove(rmate)));
                    paired = true;
                    break;
                }
            }

            if !paired {
                lmate += 1;
            } else {
                bunled += 1;
            }
        }

        Ok(bunled)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PairedEndBundler<I> {
    inner: I,
    bundles: HashMap<Vec<u8>, Bundle>,
    cache: Vec<Bundle>,
    batch: Vec<(Record, Record)>,

    bundle_every: usize,
    batch_size: usize,
}

impl<I> PairedEndBundler<I>
where
    I: LendingIterator,
    for<'iter> <<I as LendingIterator>::Item as ForLt>::Of<'iter>:
        Into<io::Result<&'iter [Record]>>,
{
    pub const DEFAULT_BUNDLE_EVERY: usize = 16;
    pub const DEFAULT_BATCH_SIZE: usize = 1024;

    pub fn new(reader: I) -> Self {
        Self {
            inner: reader,
            bundles: HashMap::new(),
            cache: Vec::new(),
            batch: Vec::new(),

            bundle_every: Self::DEFAULT_BUNDLE_EVERY,
            batch_size: Self::DEFAULT_BATCH_SIZE,
        }
    }

    fn read(&mut self) -> io::Result<usize> {
        let mut batches = 1;
        self.batch.clear();

        while let Some(batch) = self.inner.next() {
            for record in batch.into()? {
                let qname = record.name().ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Query name must be present in the BAM file",
                ))?;

                if self.bundles.contains_key(qname.as_bytes()) {
                    let bundle = self.bundles.get_mut(qname.as_bytes()).unwrap();
                    bundle.push(record.clone());
                } else {
                    let mut bundle = self.cache.pop().unwrap_or(Bundle::new());
                    bundle.push(record.clone());
                    self.bundles.insert(qname.as_bytes().to_vec(), bundle);
                }

                batches += 1;
            }

            // Run the bundling if needed
            if batches % self.bundle_every == 0 {
                self.bundles = std::mem::take(&mut self.bundles)
                    .into_iter()
                    .filter_map(|(qname, mut bundle)| {
                        let result = bundle.try_bundle(&mut self.batch);
                        if result.is_err() {
                            Some(Err(result.unwrap_err()))
                        } else if bundle.is_empty() {
                            self.cache.push(bundle);
                            None
                        } else {
                            Some(Ok((qname, bundle)))
                        }
                    })
                    .collect::<io::Result<HashMap<_, _>>>()?;
            }

            if self.batch.len() >= self.batch_size {
                break;
            }
        }

        Ok(self.batch.len())
    }
}

impl<I> LendingIterator for PairedEndBundler<I>
where
    I: LendingIterator,
    for<'iter> <<I as LendingIterator>::Item as ForLt>::Of<'iter>:
        Into<io::Result<&'iter [Record]>>,
{
    type Item = For!(<'iter> = io::Result<&'iter [(Record, Record)]>);

    fn next(self: &'_ mut Self) -> Option<<Self::Item as ForLt>::Of<'_>> {
        match self.read() {
            Ok(0) => None,
            Ok(_) => Some(Ok(&self.batch)),
            Err(err) => Some(Err(err)),
        }
    }
}
