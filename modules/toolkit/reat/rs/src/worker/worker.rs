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
use crate::selection::{Selection, Selector};
use crate::task::{Task, TaskPileup};

pub type SourceArgs<SeqId> = For!(<'args> = (&'args SeqId, usize, usize));
pub type SourceItem = For!(<'item> = io::Result<(Orientation, &'item [Record])>);
pub type DynReadSource<SeqId> = Box<dyn DynSource<Args = SourceArgs<SeqId>, Item = SourceItem>>;

pub struct Worker<SeqId, Idx: PrimUInt, Cnts: PrimUInt> {
    selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
    reference: RefReader,
    pileups: PileupCache<Idx, Cnts>,
    selection: Selection,
    min_phred: u8,
}

impl<SeqId, Idx, Cnts> Worker<SeqId, Idx, Cnts>
where
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn new(
        reference: Box<dyn IndexedReaderMutOp + Send + Sync>,
        selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
        min_phred: u8,
        size_hint: usize,
    ) -> Self {
        Self {
            selector,
            reference: RefReader::with_capacity(reference, size_hint),
            pileups: PileupCache::with_capacity(size_hint),
            selection: Selection::zeros(size_hint),
            min_phred,
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
    ) -> Result<PerOrientation<Option<TaskPileup<Idx, Cnts>>>> {
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
                    process::process(envelope, pileup.counts_mut(), record, self.min_phred)?;
                }
            }
        }

        self.finalize(task)
    }

    fn finalize(
        &mut self,
        task: &Task<SeqId, Idx>,
    ) -> Result<PerOrientation<Option<TaskPileup<Idx, Cnts>>>> {
        let mut results = PerOrientation::new(None, None, None);
        let mut refseq = Option::None;
        for (orientation, pileup) in self.pileups.initialized() {
            debug_assert!(task.envelope().envelops(pileup.interval()));

            // Skip if the pileup didn't have any coverage
            // It saves us a trip to the reference source and the selector call.
            if pileup
                .counts()
                .iter()
                .map(|x| x.coverage())
                .max()
                .unwrap_or(Cnts::zero())
                == Cnts::zero()
            {
                continue;
            }

            // Fetch the reference if needed
            let reference = match refseq {
                Some(reference) => reference,
                None => {
                    self.reference
                        .fetch(task.seqid().as_ref(), task.envelope())?;
                    refseq = Some(self.reference.reference());
                    self.reference.reference()
                }
            };

            self.selection.reset(pileup.len());
            self.selector.select(
                task.seqid(),
                orientation,
                pileup,
                reference,
                &mut self.selection,
            )?;
            task.exclude_outside_intervals(&mut self.selection)?;

            let offsets = self.selection.selected_offsets().collect::<Vec<_>>();
            if offsets.is_empty() {
                continue;
            }
            let reference = offsets
                .iter()
                .map(|offset| reference[*offset])
                .collect::<Vec<_>>();
            // SAFETY: All offsets are not empty, and they are in bounds of the pileup, which is guaranteed by the selector and the task's envelope.
            let pileup =
                unsafe { crate::pileup::SparsePileup::from_dense_unchecked(pileup, offsets) };
            results[orientation] = Some(TaskPileup::new(pileup, reference)?);
        }
        Ok(results)
    }
}
