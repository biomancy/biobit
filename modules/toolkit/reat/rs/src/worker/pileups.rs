use biobit_core_rs::loc::{IntervalOp, Orientation, PerOrientation};
use biobit_core_rs::num::PrimUInt;
use eyre::Result;

use crate::pileup::{DensePileup, Pileup};
use crate::workload::Task;

pub struct PileupCache<SeqId, Idx: PrimUInt, Cnts: PrimUInt> {
    pileups: PerOrientation<Option<DensePileup<SeqId, Idx, Cnts>>>,
    initialized: PerOrientation<bool>,
    capacity: usize,
}

impl<SeqId, Idx, Cnts> PileupCache<SeqId, Idx, Cnts>
where
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pileups: PerOrientation::new(None, None, None),
            initialized: PerOrientation::new(false, false, false),
            capacity,
        }
    }
}

impl<SeqId, Idx, Cnts> PileupCache<SeqId, Idx, Cnts>
where
    SeqId: Clone,
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn reset(&mut self) {
        self.initialized
            .apply(|_, initialized| *initialized = false);
    }

    pub fn get(
        &mut self,
        task: &Task<SeqId, Idx>,
        orientation: Orientation,
    ) -> Result<&mut DensePileup<SeqId, Idx, Cnts>> {
        if !self.initialized[orientation] {
            let pileup = &mut self.pileups[orientation];
            if let Some(pileup) = pileup {
                pileup.reset(task.seqid.clone(), task.envelope, orientation)?;
            } else {
                let length =
                    task.envelope.len().to_usize().ok_or_else(|| {
                        eyre::eyre!("Pileup interval length does not fit into usize")
                    })?;
                let capacity = self.capacity.max(length);
                let mut cnts = Pileup::with_capacity(capacity);
                cnts.reset(length);

                *pileup = Some(DensePileup::new(
                    task.seqid.clone(),
                    task.envelope,
                    orientation,
                    cnts,
                )?);
            }
            self.initialized[orientation] = true;
        }

        Ok(self.pileups[orientation]
            .as_mut()
            .expect("initialized pileup should exist"))
    }

    pub fn initialized(
        &self,
    ) -> impl Iterator<Item = (Orientation, &DensePileup<SeqId, Idx, Cnts>)> {
        self.pileups
            .iter()
            .filter(|(orientation, _)| self.initialized[*orientation])
            .map(|(orientation, pileup)| {
                (
                    orientation,
                    pileup.as_ref().expect("initialized pileup should exist"),
                )
            })
    }
}
