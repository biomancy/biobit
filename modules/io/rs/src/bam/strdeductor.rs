use noodles::bam::Record;

use biobit_core_rs::loc::Orientation;

pub trait StrDeductor {
    fn deduce(&mut self, records: &Record) -> Orientation;
}

impl<T> StrDeductor for T
where
    T: FnMut(&Record) -> Orientation,
{
    fn deduce(&mut self, records: &Record) -> Orientation {
        (self)(records)
    }
}

// impl StrDeductor for SeqLib {
//     fn deduce(&self, records: &[Record], saveto: &mut Vec<Orientation>) {
//         saveto.clear();
//         let records = records.iter();
//
//         match self {
//             SeqLib::Single { strandedness } => {
//                 match strandedness {
//                     Strandedness::Forward => {
//                         saveto.extend(
//                             records.map(|record| strandedness::deduce::se::forward(record.flags().is_reverse_complemented()))
//                         )
//                     }
//                     Strandedness::Reverse => {
//                         saveto.extend(
//                             records.map(|record| strandedness::deduce::se::reverse(record.flags().is_reverse_complemented()))
//                         )
//                     }
//                     Strandedness::Unstranded => {
//                         saveto.resize(records.len(), Orientation::Dual)
//                     }
//                 }
//             }
//             SeqLib::Paired { strandedness, orientation } => {
//                 match (strandedness, orientation) {
//                     (Strandedness::Forward, MatesOrientation::Inward) => {
//                         saveto.extend(
//                             records.map(
//                                 |record| strandedness::deduce::pe::forward(
//                                     record.flags().is_first_segment(),
//                                     record.flags().is_reverse_complemented())
//                             )
//                         )
//                     }
//                     (Strandedness::Reverse, MatesOrientation::Inward) => {
//                         saveto.extend(
//                             records.map(
//                                 |record| strandedness::deduce::pe::reverse(
//                                     record.flags().is_first_segment(),
//                                     record.flags().is_reverse_complemented())
//                             )
//                         )
//                     }
//                     _ => unimplemented!("Unsupported strandedness and mates orientation")
//                 }
//             }
//         }
//     }
// }
