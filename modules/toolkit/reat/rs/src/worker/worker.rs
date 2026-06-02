use super::pileups::PileupCache;
use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{IntervalOp, Orientation};
use biobit_core_rs::num::PrimUInt;
use biobit_core_rs::source::DynSource;
use biobit_io_rs::fasta::IndexedReaderMutOp;
use eyre::Result;
use higher_kinded_types::prelude::*;
use noodles::bam::Record;
use std::io;
use std::sync::Arc;

use super::process;
use super::reference::RefReader;
use crate::pileup::SparsePileup;
use crate::selection::{Selection, Selector};
use crate::task::Task;

pub type SourceArgs<SeqId> = For!(<'args> = (&'args SeqId, usize, usize));
pub type SourceItem = For!(<'item> = io::Result<(Orientation, &'item [Record])>);
pub type DynReadSource<SeqId> = Box<dyn DynSource<Args = SourceArgs<SeqId>, Item = SourceItem>>;

pub struct Worker<SeqId, Idx: PrimUInt, Cnts: PrimUInt> {
    selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
    reference: RefReader,
    pileups: PileupCache<SeqId, Idx, Cnts>,
    selection: Selection,
    min_phread: u8,
}

impl<SeqId, Idx, Cnts> Worker<SeqId, Idx, Cnts>
where
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn new(
        reference: Box<dyn IndexedReaderMutOp + Send + Sync>,
        selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
        min_phread: u8,
        size_hint: usize,
    ) -> Self {
        Self {
            selector,
            reference: RefReader::with_capacity(reference, size_hint),
            pileups: PileupCache::with_capacity(size_hint),
            selection: Selection::zeros(size_hint),
            min_phread,
        }
    }
}

impl<SeqId, Idx, Cnts> Worker<SeqId, Idx, Cnts>
where
    SeqId: AsRef<str> + Clone + PartialEq,
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn process(
        &mut self,
        task: &Task<SeqId, Idx>,
        sources: &mut [DynReadSource<SeqId>],
    ) -> Result<Vec<SparsePileup<SeqId, Idx, Cnts>>> {
        let envelope = task
            .envelope()
            .cast::<usize>()
            .ok_or_else(|| eyre::eyre!("task envelope does not fit into usize"))?;
        self.pileups.reset();

        for source in sources {
            let mut iter = source.fetch((task.seqid(), envelope.start(), envelope.end()))?;
            while let Some(batch) = iter.next() {
                let (orientation, records) = batch?;
                let pileup = self.pileups.get(task, orientation)?;
                debug_assert_eq!(pileup.interval(), task.envelope());
                for record in records {
                    process::process(envelope, pileup.counts_mut(), record, self.min_phread)?;
                }
            }
        }

        self.finalize(task)
    }

    fn finalize(&mut self, task: &Task<SeqId, Idx>) -> Result<Vec<SparsePileup<SeqId, Idx, Cnts>>> {
        let mut is_reference_updated = false;
        let mut results = Vec::new();
        for (_orientation, pileup) in self.pileups.initialized() {
            debug_assert!(pileup.seqid == *task.seqid());
            debug_assert!(task.envelope().envelops(pileup.interval()));

            // Fetch the reference if needed
            if !is_reference_updated {
                self.reference
                    .fetch(task.seqid().as_ref(), task.envelope())?;
                is_reference_updated = true;
            }

            self.selection.reset(pileup.len());
            self.selector
                .select(pileup, self.reference.reference(), &mut self.selection)?;
            task.exclude_outside_intervals(&mut self.selection)?;

            let offsets = self.selection.selected_offsets().collect::<Vec<_>>();
            if offsets.is_empty() {
                continue;
            }
            results.push(SparsePileup::from_dense(pileup, offsets)?);
        }
        Ok(results)
    }
}
