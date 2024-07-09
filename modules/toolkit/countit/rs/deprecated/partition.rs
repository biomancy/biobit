
use std::hash::Hash;

use derive_getters::Dissolve;
use biobit_core_rs::loc::{Contig, Orientation, Segment};
use biobit_core_rs::num::PrimInt;


#[derive(Clone, PartialEq, Eq, Debug, Dissolve)]
pub struct Partition<Ctg: Contig, Idx: PrimInt, T> {
    contig: Ctg,
    envelope: Segment<Idx>,
    data: Vec<(Segment<Idx>, Orientation, T)>,
}

impl<Ctg: Contig, Idx: PrimInt, T> Partition<Ctg, Idx, T> {
    // pub fn from_loci<'a>(
    //     loci: impl Iterator<Item=(&'a (impl LocusLike<Contig=Ctg, Idx=Idx> + 'a), T)>
    // ) -> impl Iterator<Item=Partition<Ctg, Idx, T>> + 'a
    // where
    //     Ctg: 'a,
    //     Idx: 'a,
    //     T: 'a,
    // {
    //     // let mut contigs = HashMap::new();
    //     // let mut limits = HashMap::new();
    //     //
    //     // for (locus, data) in loci {
    //     //     // Save the locus as belonging to the contig
    //     //     contigs.entry(locus.contig())
    //     //         .or_insert_with(Vec::new)
    //     //         .push((locus.segment().as_interval(), locus.orientation(), data));
    //     //
    //     //     // Update the limits of the contig
    //     //     let ctglims = limits.entry(locus.contig())
    //     //         .or_insert_with(|| (Idx::max_value(), Idx::zero()));
    //     //     ctglims.0 = ctglims.0.min(locus.segment().start());
    //     //     ctglims.1 = ctglims.1.max(locus.segment().end());
    //     // }
    //     //
    //     // // Save the partitions
    //     // contigs.into_iter()
    //     //     .map(move |(contig, data)| {
    //     //         let limits = limits.get(&contig).unwrap();
    //     //         let envelope = Segment::new(limits.0, limits.1).unwrap();
    //     //         Partition {
    //     //             contig: contig.clone(),
    //     //             envelope,
    //     //             data,
    //     //         }
    //     //     })
    //     todo!("Implement from contigs")
    // }
}
