use super::pileups::PileupCache;
use biobit_core_rs::LendingIterator;
use biobit_core_rs::loc::{IntervalOp, Orientation, PerOrientation};
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
use crate::workload::Task;

pub type SourceArgs<SeqId> = For!(<'args> = (&'args SeqId, usize, usize));
pub type SourceItem = For!(<'item> = io::Result<(Orientation, &'item [Record])>);
pub type DynReadSource<SeqId> = Box<dyn DynSource<Args = SourceArgs<SeqId>, Item = SourceItem>>;
pub type WorkerResults<SeqId, Idx, Cnts> =
    PerOrientation<Vec<(Task<SeqId, Idx>, SparsePileup<SeqId, Idx, Cnts>)>>;

pub struct Worker<SeqId, Idx: PrimUInt, Cnts: PrimUInt> {
    selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
    sources: Vec<DynReadSource<SeqId>>,
    reference: RefReader,
    pileups: PileupCache<SeqId, Idx, Cnts>,
    selection: Selection,
    results: WorkerResults<SeqId, Idx, Cnts>,
    min_phread: u8,
}

impl<SeqId, Idx, Cnts> Worker<SeqId, Idx, Cnts>
where
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn new(
        sources: Vec<DynReadSource<SeqId>>,
        reference: Box<dyn IndexedReaderMutOp + Send + Sync>,
        selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
        min_phread: u8,
        size_hint: usize,
    ) -> Self {
        Self {
            selector,
            sources,
            reference: RefReader::with_capacity(reference, size_hint),
            pileups: PileupCache::with_capacity(size_hint),
            selection: Selection::zeros(size_hint),
            results: PerOrientation::new(Vec::new(), Vec::new(), Vec::new()),
            min_phread,
        }
    }

    pub fn dissolve(self) -> WorkerResults<SeqId, Idx, Cnts> {
        self.results
    }
}

impl<SeqId, Idx, Cnts> Worker<SeqId, Idx, Cnts>
where
    SeqId: AsRef<str> + Clone + PartialEq,
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn process(&mut self, task: Task<SeqId, Idx>) -> Result<()> {
        let envelope = task
            .envelope
            .cast::<usize>()
            .ok_or_else(|| eyre::eyre!("task envelope does not fit into usize"))?;
        self.pileups.reset();

        for source in &mut self.sources {
            let mut iter = source.fetch((&task.seqid, envelope.start(), envelope.end()))?;
            while let Some(batch) = iter.next() {
                let (orientation, records) = batch?;
                let pileup = self.pileups.get(&task, orientation)?;
                debug_assert_eq!(pileup.interval(), task.envelope);
                for record in records {
                    process::process(envelope, pileup.counts_mut(), record, self.min_phread)?;
                }
            }
        }

        self.finalize(task)
    }

    fn finalize(&mut self, task: Task<SeqId, Idx>) -> Result<()> {
        let mut is_reference_updated = false;
        for (orientation, pileup) in self.pileups.initialized() {
            debug_assert!(pileup.seqid == task.seqid);
            debug_assert!(task.envelope.envelops(&pileup.interval()));

            // Fetch the reference if needed
            if !is_reference_updated {
                self.reference.fetch(task.seqid.as_ref(), task.envelope)?;
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
            self.results[orientation]
                .push((task.clone(), SparsePileup::from_dense(pileup, offsets)?))
        }
        Ok(())
    }
}
