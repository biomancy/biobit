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

use super::config::Config;
use super::engine::{Comparison, Engine};
use super::result::Ripped;

pub struct Ripper<Ctg, Idx, Cnts, SmplTag, CmpTag, Src>
where
    Idx: PrimInt + Send,
    Cnts: Float + Send,
    Ctg: Contig + Send,
    SmplTag: PartialOrd + Ord,
{
    pool: ThreadPool,
    engine: Engine<Ctg, Idx, Cnts>,
    samples: BTreeMap<SmplTag, Vec<Src>>,
    comparison: Vec<Comparison<Idx, Cnts, CmpTag, Src>>,
    partitions: Vec<(Ctg, Idx, Idx)>,
}

impl<Ctg, Idx, Cnts, SmplTag, CmpTag, Src> Ripper<Ctg, Idx, Cnts, SmplTag, CmpTag, Src>
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
            partitions: Vec::new(),
        }
    }

    pub fn add_partition(&mut self, contig: Ctg, start: Idx, end: Idx) -> &mut Self {
        self.partitions.push((contig, start, end));
        self
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
        config: Config<Idx, Cnts>,
    ) -> Result<&mut Self> {
        let signal = self.get_sources(signal)?;
        let control = self.get_sources(control)?;

        self.comparison
            .push(Comparison::new(tag, signal, control, config));
        Ok(self)
    }

    pub fn run(&mut self) -> Result<Vec<Ripped<Ctg, Idx, Cnts, CmpTag>>> {
        let result = self.engine.run(
            &mut self.pool,
            std::mem::take(&mut self.partitions),
            std::mem::take(&mut self.comparison),
        );

        self.samples.clear();
        self.engine.reset();
        result
    }

    // pub fn run(&mut self) -> Result<Vec<Counts<Ctg, Idx, Cnts, Data, Tag>>> {
    //     // Index the annotation
    //     let itrees = self.pool.install(|| {
    //         std::mem::take(&mut self.annotations)
    //             .into_iter()
    //             .par_bridge()
    //             .map(|((contig, orientation), data)| {
    //                 let mut tree = LapperBuilder::new();
    //                 for (ind, segments) in data {
    //                     for segment in segments {
    //                         tree = tree.add(&segment, ind);
    //                     }
    //                 }
    //                 (contig, orientation, tree)
    //             })
    //             .collect::<Vec<_>>()
    //     });
    //
    //     let mut gindex = GenomicIndex::new();
    //     for (contig, orientation, tree) in itrees {
    //         gindex.set(contig, orientation, tree.build());
    //     }
    //
    //     // Run the counting
    //     let result = self.engine.run(
    //         &mut self.pool,
    //         self.data.len(),
    //         &self.sources,
    //         &gindex,
    //         &self.partitions,
    //     )?;
    //
    //     Ok(result
    //         .into_iter()
    //         .zip(self.tags.drain(..))
    //         .map(|((cnts, stats), tag)| Counts::new(tag, self.data.clone(), cnts, stats))
    //         .collect())
    // }

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
