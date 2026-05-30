use biobit_core_rs::num::PrimUInt;
use eyre::Result;

use crate::dna::Reference;
use crate::pileup::DensePileup;
use crate::selection::Selection;

pub trait Selector<SeqId, Idx: PrimUInt, Cnts: PrimUInt> {
    fn select(
        &self,
        pileup: &DensePileup<SeqId, Idx, Cnts>,
        reference: &[Reference],
        selection: &mut Selection,
    ) -> Result<()>;
}
