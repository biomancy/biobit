use ahash::HashMap;
use biobit_collections_rs::interval_tree::Bits;
use biobit_core_rs::loc::{Contig, Interval, IntervalOp, PerOrientation};
use biobit_core_rs::num::PrimInt;
use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;
use log;

// Partition is not 'per orientation' because BAM files are indexed only by contig and position
// Contig + Orientation indexing will be the only supported indexing scheme in the future
#[derive(Debug, Dissolve, Constructor, Getters)]
pub struct Partition<Ctg, Idx: PrimInt> {
    // Coordinates of the partition
    contig: Ctg,
    interval: Interval<Idx>,
    // Elements that belong to the partition
    index: PerOrientation<Bits<Idx, usize>>, // Annotation index (values are indices in the elements vector)
    eltinds: Vec<usize>, // Global index of each element (to aggregate counts across partitions)
}

impl<Ctg: Contig, Idx: PrimInt> Partition<Ctg, Idx> {
    pub fn build(
        contig: Ctg,
        mut partitions: Vec<Interval<Idx>>,
        candidates: PerOrientation<Vec<(usize, Vec<Interval<Idx>>)>>,
    ) -> Vec<Self> {
        // Merge all partitions to avoid overlapping queries downstream
        let before = partitions.len();
        let partitions = Interval::merge(&mut partitions);
        let after = partitions.len();

        // Report if there were overlapping partitions
        if before != after {
            let diff = before - after;
            log::warn!("Merged {diff} overlapping partitions for contig {contig:?}");
        }

        // Build the partitions index
        let prtindex = Bits::new(partitions.iter().cloned().enumerate());

        // Map elements to partitions per orientation
        let mut elements_per_partition =
            vec![HashMap::<usize, PerOrientation<Vec<_>>>::default(); partitions.len()];
        for (orientation, candidates) in candidates.into_iter() {
            for (elind, mut segments) in candidates {
                for segment in Interval::merge(&mut segments) {
                    for (_, prtind) in prtindex.query(segment.start(), segment.end()) {
                        elements_per_partition[*prtind]
                            .entry(elind)
                            .or_default()
                            .get_mut(orientation)
                            .push(segment);
                    }
                }
            }
        }

        // Build final partitions
        elements_per_partition
            .into_iter()
            .zip(partitions)
            .map(|(mapped, segment)| {
                let mut index: PerOrientation<Vec<_>> = PerOrientation::default();
                let mut elements = Vec::with_capacity(mapped.len());
                for (elind, orientations) in mapped {
                    let ind = elements.len();
                    elements.push(elind);

                    for (orientation, segments) in orientations {
                        for segment in segments {
                            index.get_mut(orientation).push((ind, segment));
                        }
                    }
                }

                // Build the partition index for each orientation
                let mut itree = PerOrientation::default();
                for (orientation, elements) in index {
                    *itree.get_mut(orientation) = Bits::new(elements.into_iter());
                }

                Partition::new(contig.clone(), segment, itree, elements)
            })
            .collect()
    }
}
