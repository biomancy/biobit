use crate::rigid::partition::Partition;
use crate::rigid::Engine;
use ahash::AHashMap;
use biobit_core_rs::loc::{AsSegment, Orientation, PerOrientation, Segment};
use biobit_core_rs::{
    loc::Contig,
    num::{Float, PrimInt},
};
use rayon::prelude::*;
use rayon::ThreadPool;
use thread_local::ThreadLocal;

pub struct EngineBuilder<Ctg: Contig, Idx: PrimInt, Elt> {
    elements: Vec<Elt>,
    annotation: AHashMap<Ctg, PerOrientation<Vec<(usize, Vec<Segment<Idx>>)>>>,
    partitions: AHashMap<Ctg, Vec<Segment<Idx>>>,
    thread_pool: Option<ThreadPool>,
}

impl<Ctg: Contig, Idx: PrimInt, Elt> Default for EngineBuilder<Ctg, Idx, Elt> {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            annotation: AHashMap::new(),
            partitions: AHashMap::new(),
            thread_pool: None,
        }
    }
}

impl<Ctg: Contig, Idx: PrimInt, Elt> EngineBuilder<Ctg, Idx, Elt>
where
    Elt: Send + Sync,
{
    pub fn add_elements(
        mut self,
        elements: impl Iterator<Item = (Elt, Vec<(Ctg, Orientation, Vec<Segment<Idx>>)>)>,
    ) -> Self {
        for (element, segments) in elements {
            let ind = self.elements.len();
            self.elements.push(element);
            for (contig, orientation, segments) in segments {
                self.annotation
                    .entry(contig)
                    .or_default()
                    .get_mut(orientation)
                    .push((ind, segments));
            }
        }
        self
    }

    pub fn add_partitions(mut self, partitions: impl Iterator<Item = (Ctg, Segment<Idx>)>) -> Self {
        for (contig, segment) in partitions {
            self.partitions.entry(contig).or_default().push(segment);
        }
        self
    }

    pub fn set_thread_pool(mut self, pool: ThreadPool) -> Self {
        self.thread_pool = Some(pool);
        self
    }

    pub fn _build(&mut self) -> Vec<Partition<Ctg, Idx>> {
        // Prepare the workload for each thread
        let mut workload = Vec::new();
        for (contig, mut segments) in std::mem::take(&mut self.partitions).into_iter() {
            // Select elements inside the partition
            let elements = self.annotation.remove(&contig).unwrap_or_default();
            workload.push((contig, segments, elements));
        }

        // Prepare partitions
        let partitions: Vec<_> = workload
            .into_par_iter()
            .map(|(contig, segments, elements)| Partition::build(contig, segments, elements))
            .flatten()
            .collect();

        // Identify and report unused elements
        let mut used = vec![false; self.elements.len()];
        for prt in &partitions {
            for ind in prt.eltinds() {
                used[*ind] = true;
            }
        }
        let unused: Vec<_> = used
            .into_iter()
            .enumerate()
            .filter_map(|(ind, used)| if used { None } else { Some(ind) })
            .collect();

        if !unused.is_empty() {
            log::warn!(
                "Elements with the following indices (N={}) were not part of any partition: {:?}",
                unused.len(),
                unused
            );
        }

        partitions
    }

    pub fn build<Cnts>(mut self) -> Engine<Ctg, Idx, Cnts, Elt>
    where
        Cnts: Float + Send,
    {
        let (pool, partitions) = match self.thread_pool.take() {
            Some(pool) => {
                let partitions = pool.install(|| self._build());
                (Some(pool), partitions)
            }
            None => (None, self._build()),
        };
        Engine::new(pool, self.elements, ThreadLocal::new(), partitions)
    }
}
