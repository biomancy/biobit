use std::io;

use ahash::AHashMap;
use eyre::Result;
use higher_kinded_types::prelude::*;
use rayon::prelude::*;
use rayon::ThreadPool;

use biobit_collections_rs::genomic_index::GenomicIndex;
use biobit_collections_rs::interval_tree::{Builder, LapperBuilder};
use biobit_core_rs::{
    loc::{Contig, Locus},
    num::{Float, PrimInt, PrimUInt},
};
use biobit_core_rs::loc::{Orientation, Segment};
use biobit_core_rs::source::Source;
use biobit_io_rs::bam::SegmentedAlignment;

use crate::engine::Engine;

use super::result::Counts;

pub struct CountIt<Ctg, Idx, Cnts, Data, Tag, Src>
where
    Idx: PrimInt + Send,
    Cnts: Float + Send,
    Ctg: Contig + Send,
{
    pool: ThreadPool,
    engine: Engine<Ctg, Idx, Cnts>,

    data: Vec<Data>,
    partitions: Vec<Locus<Ctg, Idx>>,

    tags: Vec<Tag>,
    sources: Vec<Src>,

    annotations: AHashMap<(Ctg, Orientation), Vec<(usize, Vec<Segment<Idx>>)>>,
}

impl<Ctg, Idx, Cnts, Data, Tag, Src> CountIt<Ctg, Idx, Cnts, Data, Tag, Src>
where
    Idx: PrimUInt + Send + Sync + 'static,
    Cnts: Float + Send,
    Ctg: Contig + Send + Sync,
    Data: Clone,
    Src: Source<
        Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
        Item = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<Idx>>),
    >,
{
    pub fn new(pool: ThreadPool) -> Self {
        Self {
            pool,
            engine: Engine::default(),
            data: Vec::new(),
            partitions: Vec::new(),
            tags: Vec::new(),
            sources: Vec::new(),
            annotations: AHashMap::new(),
        }
    }

    pub fn add_annotation(
        &mut self,
        item: Data,
        locations: impl Iterator<Item = (Ctg, Orientation, impl Iterator<Item = Segment<Idx>>)>,
    ) {
        let ind = self.data.len();

        self.data.push(item);
        for (contig, orientation, segments) in locations {
            self.annotations
                .entry((contig, orientation))
                .or_default()
                .push((ind, segments.collect()));
        }
    }

    pub fn add_source(&mut self, tag: Tag, source: Src) {
        self.tags.push(tag);
        self.sources.push(source);
    }

    pub fn add_sources(&mut self, sources: impl Iterator<Item = (Tag, Src)>) {
        for src in sources {
            self.add_source(src.0, src.1);
        }
    }

    pub fn add_partition(&mut self, partition: Locus<Ctg, Idx>) {
        self.partitions.push(partition);
    }

    pub fn run(&mut self) -> Result<Vec<Counts<Ctg, Idx, Cnts, Data, Tag>>> {
        // Index the annotation
        let itrees = self.pool.install(|| {
            std::mem::take(&mut self.annotations)
                .into_iter()
                .par_bridge()
                .map(|((contig, orientation), data)| {
                    let mut tree = LapperBuilder::new();
                    for (ind, segments) in data {
                        for segment in segments {
                            tree = tree.add(&segment, ind);
                        }
                    }
                    (contig, orientation, tree)
                })
                .collect::<Vec<_>>()
        });

        let mut gindex = GenomicIndex::new();
        for (contig, orientation, tree) in itrees {
            gindex.set(contig, orientation, tree.build());
        }

        // Run the counting
        let result = self.engine.run(
            &mut self.pool,
            self.data.len(),
            &self.sources,
            &gindex,
            &self.partitions,
        )?;

        Ok(result
            .into_iter()
            .zip(self.tags.drain(..))
            .map(|((cnts, stats), tag)| Counts::new(tag, self.data.clone(), cnts, stats))
            .collect())
    }
}
