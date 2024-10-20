use std::io;

use ahash::AHashMap;
use eyre::Result;
use higher_kinded_types::prelude::*;
use rayon::prelude::*;
use rayon::ThreadPool;

use biobit_collections_rs::interval_tree::{BitsBuilder, Builder};
use biobit_core_rs::loc::{Orientation, Segment};
use biobit_core_rs::source::Source;
use biobit_core_rs::{
    loc::{Contig, Locus},
    num::{Float, PrimInt, PrimUInt},
};
use biobit_io_rs::bam::SegmentedAlignment;

use crate::engine::Engine;

use super::result::Counts;

// Vectors of target elements.
// Each vector has the same length and correspond to a single element.
pub struct VecOfTargets<Ctg, Idx: PrimInt, Elt, EltTag> {
    elements: Vec<Elt>, // Target elements supplied by the user. No requirements, reported as is in the results.
    tags: Vec<EltTag>, // Non-unique tags used to perform on-the-fly resolution of overlapping elements
    // Segments that belong to each element
    // (ctg, orient) -> [(element index, element segments), ...]
    segments: AHashMap<(Ctg, Orientation), Vec<(usize, Vec<Segment<Idx>>)>>,
}

// Vector of alignment sources
pub struct VecOfSources<SrcTag, Src> {
    inds: Vec<SrcTag>, // User-supplied IDs for each source. No requirements, reported as is in the results.
    sources: Vec<Src>, // Alignment sources
}

pub struct CountIt<Ctg, Idx, Cnts, Elt, EltTag, SrcTag, Src>
where
    Idx: PrimInt + Send,
    Cnts: Float + Send,
    Ctg: Contig + Send,
{
    // ThreadPool used to run the counting. If not provided, a default one will be used.
    pool: Option<ThreadPool>,

    // Target elements for counting
    targets: VecOfTargets<Ctg, Idx, Elt, EltTag>,

    // Target regions that will be processed during counting
    // They might or might not overlap with the target elements
    partitions: Vec<(Ctg, Orientation, Segment<Idx>)>,

    // Alignment sources
    sources: VecOfSources<SrcTag, Src>,

    // Internal processing engine
    engine: Engine<Ctg, Idx, Cnts>,
}

impl<Ctg, Idx, Cnts, Elt, EltTag, SrcTag, Src> CountIt<Ctg, Idx, Cnts, Elt, EltTag, SrcTag, Src>
where
    Idx: PrimUInt + Send + Sync + 'static,
    Cnts: Float + Send,
    Ctg: Contig + Send + Sync,
    Elt: Send,
    EltTag: Send,
    SrcTag: Send,
    Src: Source<
        Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
        Item = For!(<'iter> = io::Result<&'iter mut SegmentedAlignment<Idx>>),
    >,
{
    pub fn new(pool: Option<ThreadPool>) -> Self {
        Self {
            pool,
            targets: VecOfTargets {
                elements: Vec::new(),
                tags: Vec::new(),
                segments: AHashMap::new(),
            },
            partitions: Vec::new(),
            sources: VecOfSources {
                inds: Vec::new(),
                sources: Vec::new(),
            },
            engine: Engine::default(),
        }
    }

    pub fn add_annotation(
        &mut self,
        element: Elt,
        tag: EltTag,
        segments: impl Iterator<Item = (Ctg, Orientation, impl Iterator<Item = Segment<Idx>>)>,
    ) {
        let ind = self.targets.elements.len();
        self.targets.elements.push(element);
        self.targets.tags.push(tag);
        for (contig, orientation, segments) in segments {
            self.targets
                .segments
                .entry((contig, orientation))
                .or_default()
                .push((ind, segments.collect()));
        }
    }

    pub fn add_source(&mut self, ind: SrcTag, source: Src) {
        self.sources.inds.push(ind);
        self.sources.sources.push(source);
    }

    pub fn add_partition(&mut self, partition: impl Into<(Ctg, Orientation, Segment<Idx>)>) {
        self.partitions.push(partition.into());
    }

    pub fn _run(&mut self) -> Result<Vec<Counts<Ctg, Idx, Cnts, Elt, SrcTag>>> {
        // Index the annotation
        let itrees = rayon::scope(|_| {
            std::mem::take(&mut self.targets.segments)
                .into_iter()
                .par_bridge()
                .map(|((contig, orientation), data)| {
                    let mut tree = BitsBuilder::default();
                    for (ind, mut segments) in data {
                        segments = Segment::merge(&mut segments);
                        for segment in segments {
                            tree = tree.addi(&segment, ind);
                        }
                    }
                    (contig, orientation, tree.build())
                })
                .collect::<Vec<_>>()
        });

        // let mut gindex = Bundle::new();
        // for (contig, orientation, tree) in itrees {
        //     gindex.set((contig, orientation), tree);
        // }

        // // Run the counting
        // let result = self.engine.run(
        //     self.data.len(),
        //     &self.sources,
        //     &gindex,
        //     &self.partitions,
        // )?;

        // Ok(result
        //     .into_iter()
        //     .zip(self.tags.drain(..))
        //     .map(|((cnts, stats), tag)| Counts::new(tag, self.data.clone(), cnts, stats))
        //     .collect())
        todo!("Implement CountIt::_run")
    }

    pub fn run(&mut self) -> Result<Vec<Counts<Ctg, Idx, Cnts, Elt, SrcTag>>> {
        match self.pool.take() {
            // Pool is behind the Arc, so we can clone it safely
            Some(pool) => {
                let result = pool.install(|| self._run());
                self.pool = Some(pool);
                result
            }
            None => rayon::scope(|s| self._run()),
        }
    }
}
