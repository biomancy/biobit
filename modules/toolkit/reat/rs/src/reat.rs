use std::collections::BTreeMap;
use std::sync::Arc;

use biobit_core_rs::num::PrimUInt;
use biobit_core_rs::source::Source;
use biobit_io_rs::fasta::IndexedReaderMutOp;
use eyre::{Result, eyre};
use rayon::ThreadPool;

use crate::engine::{Analysis, Engine, ReferenceFactory};
use crate::result::SelectedPileup;
use crate::selection::Selector;
use crate::worker::{SourceArgs, SourceItem};
use crate::workload::Workload;

pub struct Reat<SeqId, Idx, Cnts, SmplTag, AnalysisTag, Src>
where
    Idx: PrimUInt + Send,
    Cnts: PrimUInt + Send,
    SeqId: Send,
    SmplTag: PartialOrd + Ord,
{
    pool: ThreadPool,
    reference_factory: Arc<ReferenceFactory>,
    engine: Engine<SeqId, Idx, Cnts>,
    samples: BTreeMap<SmplTag, Vec<Src>>,
    analyses: Vec<Analysis<SeqId, Idx, Cnts, AnalysisTag, Src>>,
}

impl<SeqId, Idx, Cnts, SmplTag, AnalysisTag, Src> Reat<SeqId, Idx, Cnts, SmplTag, AnalysisTag, Src>
where
    SeqId: AsRef<str> + Clone + Default + Ord + PartialEq + Send + Sync + 'static,
    Idx: PrimUInt + Send + Sync + 'static,
    Cnts: PrimUInt + Send + Sync + 'static,
    SmplTag: PartialOrd + Ord,
    Src: Source<Args = SourceArgs<SeqId>, Item = SourceItem> + 'static,
{
    pub fn new<F>(pool: ThreadPool, reference_factory: F) -> Self
    where
        F: Fn() -> Result<Box<dyn IndexedReaderMutOp + Send + Sync>> + Send + Sync + 'static,
    {
        Self::with_reference_factory(pool, Arc::new(reference_factory))
    }

    pub fn with_reference_factory(
        pool: ThreadPool,
        reference_factory: Arc<ReferenceFactory>,
    ) -> Self {
        Self {
            pool,
            reference_factory,
            engine: Engine::default(),
            samples: BTreeMap::new(),
            analyses: Vec::new(),
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

    pub fn add_analysis<Sel>(
        &mut self,
        tag: AnalysisTag,
        sample: &SmplTag,
        workload: Workload<SeqId, Idx>,
        selector: Sel,
        min_phred: u8,
    ) -> Result<&mut Self>
    where
        Sel: Selector<SeqId, Idx, Cnts> + Send + Sync + 'static,
    {
        self.add_analysis_with_selector(tag, sample, workload, Arc::new(selector), min_phred)
    }

    pub fn add_analysis_with_selector(
        &mut self,
        tag: AnalysisTag,
        sample: &SmplTag,
        workload: Workload<SeqId, Idx>,
        selector: Arc<dyn Selector<SeqId, Idx, Cnts> + Send + Sync>,
        min_phred: u8,
    ) -> Result<&mut Self> {
        let sources = self.get_sources(sample)?;
        self.analyses
            .push(Analysis::new(tag, sources, workload, selector, min_phred));
        Ok(self)
    }

    pub fn reset(&mut self) {
        self.samples.clear();
        self.analyses.clear();
        self.engine.reset();
    }

    pub fn run(&mut self) -> Result<Vec<SelectedPileup<SeqId, Idx, Cnts, AnalysisTag>>> {
        let result = self.engine.run(
            &mut self.pool,
            std::mem::take(&mut self.analyses),
            Arc::clone(&self.reference_factory),
        );
        self.reset();
        result
    }

    fn get_sources(&self, tag: &SmplTag) -> Result<Vec<Src>> {
        let sources = self
            .samples
            .get(tag)
            .ok_or_else(|| eyre!("Unknown sample tag"))?
            .iter()
            .map(dyn_clone::clone)
            .collect();
        Ok(sources)
    }
}
