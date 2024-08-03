use std::collections::BTreeMap;
use std::io;

use eyre::{eyre, Result};
use higher_kinded_types::prelude::*;
use rayon::ThreadPool;

use biobit_core_rs::{
    loc::Contig,
    num::{Float, PrimInt, PrimUInt},
};
use biobit_core_rs::source::Source;
use biobit_io_rs::bam::SegmentedAlignment;

use crate::workload::Workload;

use super::engine::{Comparison, Engine};
use super::result::Harvest;

pub struct Reaper<Ctg, Idx, Cnts, SmplTag, CmpTag, Src>
where
    Idx: PrimInt + Send,
    Cnts: Float + Send,
    Ctg: Contig + Send,
    SmplTag: PartialOrd + Ord,
{
    pool: ThreadPool,
    engine: Engine<Ctg, Idx, Cnts>,
    samples: BTreeMap<SmplTag, Vec<Src>>,
    comparison: Vec<Comparison<Ctg, Idx, Cnts, CmpTag, Src>>,
}

impl<Ctg, Idx, Cnts, SmplTag, CmpTag, Src> Reaper<Ctg, Idx, Cnts, SmplTag, CmpTag, Src>
where
    Idx: PrimUInt + Send + Sync,
    Cnts: Float + Send + Sync,
    Ctg: Contig + Send + Sync,
    SmplTag: PartialOrd + Ord,
    Src: Source<
        Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
        Item = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<Idx>>),
    >,
{
    pub fn new(pool: ThreadPool) -> Self {
        Self {
            pool,
            engine: Engine::default(),
            samples: BTreeMap::new(),
            comparison: Vec::new(),
        }
    }

    pub fn add_source(&mut self, tag: SmplTag, source: Src) -> &mut Self {
        self.samples.entry(tag).or_default().push(source);
        self
    }

    pub fn add_sources(&mut self, tag: SmplTag, sources: Vec<Src>) -> &mut Self {
        self.samples.entry(tag).or_default().extend(sources);
        self
    }

    pub fn add_comparison(
        &mut self,
        tag: CmpTag,
        signal: &SmplTag,
        control: &SmplTag,
        workload: Workload<Ctg, Idx, Cnts>,
    ) -> Result<&mut Self> {
        let signal = self.get_sources(signal)?;
        let control = self.get_sources(control)?;

        self.comparison
            .push(Comparison::new(tag, signal, control, workload));
        Ok(self)
    }

    pub fn reset(&mut self) {
        self.samples.clear();
        self.comparison.clear();
        self.engine.reset();
    }

    pub fn run(&mut self) -> Result<Vec<Harvest<Ctg, Idx, Cnts, CmpTag>>> {
        let result = self
            .engine
            .run(&mut self.pool, std::mem::take(&mut self.comparison));
        self.reset();

        result
    }

    fn get_sources(&self, tag: &SmplTag) -> Result<Vec<Src>> {
        let sources = self
            .samples
            .get(tag)
            .ok_or_else(|| eyre!("Unknown sample tag"))?
            .iter()
            .map(|x| dyn_clone::clone(x))
            .collect();
        Ok(sources)
    }
}
