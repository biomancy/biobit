use crate::core::mismatches::{Batch, Builder, MismatchesVec};
use crate::core::rpileup::noodles::HTSPileupEngine;
use noodles::bam::record::Record;

use crate::core::rpileup::{ReadsCollider, ReadsCollidingEngine};

pub trait Runner<'runner, T: MismatchesVec> {
    type Workload;

    fn run(&'runner mut self, workload: Self::Workload) -> Option<Batch<T>>;
}

#[derive(Clone)]
pub struct REATRunner<NCounter, MismatchesBuilder>
where
    for<'a> NCounter: ReadsCollider<'a, Record>,
{
    pileuper: HTSPileupEngine<NCounter>,
    mmbuilder: MismatchesBuilder,
}

impl<NCounter, MismatchesBuilder> REATRunner<NCounter, MismatchesBuilder>
where
    for<'a> NCounter: ReadsCollider<'a, Record>,
{
    pub fn new(mmbuilder: MismatchesBuilder, pileuper: HTSPileupEngine<NCounter>) -> Self {
        Self {
            pileuper,
            mmbuilder,
        }
    }
}

impl<'runner, NCounter, MBuilder> Runner<'runner, MBuilder::Out> for REATRunner<NCounter, MBuilder>
where
    for<'a> NCounter: ReadsCollider<'a, Record>,
    MBuilder: Builder<
            'runner,
            SourceCounts = <NCounter as ReadsCollider<'runner, Record>>::ColliderResult,
        >,
{
    type Workload = <NCounter as ReadsCollider<'runner, Record>>::Workload;

    fn run(&'runner mut self, workload: Self::Workload) -> Option<Batch<MBuilder::Out>> {
        self.pileuper.run(workload);

        let ncounts = match self.pileuper.result() {
            Some(x) => x,
            None => return None,
        };

        let batch = self.mmbuilder.build(ncounts);

        // Final hooks
        // self.hook.on_finish(&mut batch);
        Some(batch)
    }
}
