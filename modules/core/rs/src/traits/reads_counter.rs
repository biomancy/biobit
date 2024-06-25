use std::collections::HashMap;
use std::hash::Hash;

use derive_getters::Dissolve;

use crate::loc::{Contig, Interval, LikeInterval, LikeLocus, Orientation};
use crate::num::PrimInt;

pub trait Result {}


pub trait Counter {
    // Primary types for coordinates
    type Ctg: Contig;
    type Idx: PrimInt;
    // Data and read sources
    type Source;
    type Data: Hash + PartialEq;
    // Result type
    type Result: Result;

    fn count(&mut self, partition: Vec<Partition<Self::Ctg, Self::Idx, Self::Data>>);
    fn results(&self) -> &[Self::Result];
    fn reset(&mut self) -> Vec<Self::Result>;
}


#[derive(Clone, PartialEq, Eq, Debug, Dissolve)]
pub struct Partition<Ctg: Contig, Idx: PrimInt, T> {
    contig: Ctg,
    envelope: Interval<Idx>,
    data: Vec<(Interval<Idx>, Orientation, T)>,
}

impl<Ctg: Contig, Idx: PrimInt, T> Partition<Ctg, Idx, T> {
    pub fn from_loci<'a>(
        loci: impl Iterator<Item=(&'a (impl LikeLocus<Ctg=Ctg, Idx=Idx> + 'a), T)>
    ) -> impl Iterator<Item=Partition<Ctg, Idx, T>> + 'a
    where
        Ctg: 'a,
        Idx: 'a,
        T: 'a,
    {
        let mut contigs = HashMap::new();
        let mut limits = HashMap::new();

        for (locus, data) in loci {
            // Save the locus as belonging to the contig
            contigs.entry(locus.contig())
                .or_insert_with(Vec::new)
                .push((locus.interval().as_interval(), locus.orientation(), data));

            // Update the limits of the contig
            let ctglims = limits.entry(locus.contig())
                .or_insert_with(|| (Idx::max_value(), Idx::zero()));
            ctglims.0 = ctglims.0.min(locus.interval().start());
            ctglims.1 = ctglims.1.max(locus.interval().end());
        }

        // Save the partitions
        contigs.into_iter()
            .map(move |(contig, data)| {
                let limits = limits.get(&contig).unwrap();
                let envelope = Interval::new(limits.0, limits.1).unwrap();
                Partition {
                    contig: contig.clone(),
                    envelope,
                    data,
                }
            })
    }
}
