use bitcode::{Decode, Encode};
use derive_getters::Dissolve;
use derive_more::Constructor;
use eyre::Result;

use biobit_collections_rs::rle_vec;
use biobit_collections_rs::rle_vec::{Identical, RleVec};
use biobit_core_rs::num::{Float, PrimInt, PrimUInt};

#[derive(Encode, Decode, Clone, PartialEq, Default, Debug, Constructor, Dissolve)]
pub struct Scaling<Cnts: Float> {
    pub signal: Cnts,
    pub control: Cnts,
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, Default, Dissolve)]
pub struct Enrichment<Cnts: Float> {
    // Scaling is left here intentionally.
    // In the future I might want to do per-step ops with a higher precision and then
    // downcast the results to lower precision. Fusing scaling into that would be great.
    pub scaling: Scaling<Cnts>,
}

impl<Cnts: Float> Enrichment<Cnts> {
    pub fn new() -> Self {
        Enrichment::default()
    }

    pub fn set_scaling(&mut self, signal: Cnts, control: Cnts) -> &mut Self {
        self.scaling = Scaling { signal, control };
        self
    }

    pub fn calculate<Idx: PrimInt, Len: PrimUInt, I: Identical<Cnts>>(
        &self,
        signal: &RleVec<Cnts, Len, I>,
        control: &RleVec<Cnts, Len, I>,
        identical: I,
        buffer: RleVec<Cnts, Len, I>,
    ) -> Result<RleVec<Cnts, Len, I>> {
        let result = rle_vec::merge2(signal, control)
            .with_identical(identical)
            .with_merge2(rle_vec::Merge2Fn::new(
                |_| unreachable!("This should never be called"),
                move |&signal, &control| {
                    if signal == Cnts::zero() {
                        signal
                    } else {
                        (signal * self.scaling.signal) / (control * self.scaling.control)
                    }
                },
            ))
            .save_to(buffer)
            .run()?;
        Ok(result)
    }
}
