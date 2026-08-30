#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::{Dissolve, Getters};
use eyre::{Result, eyre};

use biobit_collections_rs::rle_vec;
use biobit_collections_rs::rle_vec::{Identical, RleVec};
use biobit_core_rs::num::{Float, PrimInt, PrimUInt};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Debug, Getters, Dissolve)]
pub struct Scaling<Cnts: Float> {
    signal: Cnts,
    control: Cnts,
}

impl<Cnts: Float> Scaling<Cnts> {
    pub fn new(signal: Cnts, control: Cnts) -> Result<Self> {
        if !signal.is_finite() || signal <= Cnts::zero() {
            return Err(eyre!("Signal scaling must be finite and greater than zero"));
        }
        if !control.is_finite() || control <= Cnts::zero() {
            return Err(eyre!(
                "Control scaling must be finite and greater than zero"
            ));
        }
        Ok(Self { signal, control })
    }
}

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Debug, Dissolve)]
pub struct Enrichment<Cnts: Float> {
    // Scaling is left here intentionally.
    // In the future I might want to do per-step ops with a higher precision and then
    // downcast the results to lower precision. Fusing scaling into that would be great.
    pub scaling: Scaling<Cnts>,
}

impl<Cnts: Float> Default for Scaling<Cnts> {
    fn default() -> Self {
        Self {
            signal: Cnts::one(),
            control: Cnts::one(),
        }
    }
}

impl<Cnts: Float> Default for Enrichment<Cnts> {
    fn default() -> Self {
        Self {
            scaling: Scaling::default(),
        }
    }
}

impl<Cnts: Float> Enrichment<Cnts> {
    pub fn new() -> Self {
        Enrichment::default()
    }

    pub fn set_scaling(&mut self, signal: Cnts, control: Cnts) -> Result<&mut Self> {
        self.scaling = Scaling::new(signal, control)?;
        Ok(self)
    }

    pub fn calculate<Idx: PrimInt, Len: PrimUInt, I: Identical<Cnts>>(
        &self,
        signal: &RleVec<Cnts, Len, I>,
        control: &RleVec<Cnts, Len, I>,
        identical: I,
        buffer: RleVec<Cnts, Len, I>,
    ) -> Result<RleVec<Cnts, Len, I>> {
        rle_vec::merge(signal, control)
            .with_identical(identical)
            .with_merge_fns(
                |_| -> Result<Cnts> { Err(eyre!("Signal and control must cover the same length")) },
                |&signal, &control| -> Result<Cnts> {
                    if !signal.is_finite()
                        || !control.is_finite()
                        || signal < Cnts::zero()
                        || control < Cnts::zero()
                    {
                        return Err(eyre!(
                            "Enrichment inputs must contain finite, non-negative values"
                        ));
                    }
                    if signal == Cnts::zero() {
                        Ok(signal)
                    } else if control == Cnts::zero() {
                        Err(eyre!(
                            "Control must be positive wherever signal is positive"
                        ))
                    } else {
                        let enrichment =
                            (signal * self.scaling.signal) / (control * self.scaling.control);
                        if enrichment.is_finite() {
                            Ok(enrichment)
                        } else {
                            Err(eyre!("Calculated enrichment is not finite"))
                        }
                    }
                },
            )
            .save_to(buffer)
            .build()?
            .run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestRle = RleVec<f64, u32, fn(&f64, &f64) -> bool>;

    fn identical(first: &f64, second: &f64) -> bool {
        first == second
    }

    fn rle(values: &[f64]) -> TestRle {
        TestRle::builder(identical as fn(&f64, &f64) -> bool)
            .with_dense_values(values)
            .unwrap()
            .build()
    }

    fn dense(rle: &TestRle) -> Vec<f64> {
        rle.runs()
            .flat_map(|(value, length)| std::iter::repeat_n(*value, *length as usize))
            .collect()
    }

    #[test]
    fn default_scaling_calculates_a_ratio() -> Result<()> {
        let result = Enrichment::new().calculate::<usize, _, _>(
            &rle(&[2.0, 0.0, 6.0]),
            &rle(&[1.0, 0.0, 3.0]),
            identical as fn(&f64, &f64) -> bool,
            rle(&[]),
        )?;

        assert_eq!(dense(&result), vec![2.0, 0.0, 2.0]);
        Ok(())
    }

    #[test]
    fn rejects_zero_control_for_positive_signal() {
        let result = Enrichment::new().calculate::<usize, _, _>(
            &rle(&[1.0]),
            &rle(&[0.0]),
            identical as fn(&f64, &f64) -> bool,
            rle(&[]),
        );
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_inputs_and_configuration() {
        let mut enrichment = Enrichment::<f64>::new();
        assert!(enrichment.set_scaling(0.0, 1.0).is_err());
        assert!(enrichment.set_scaling(1.0, f64::NAN).is_err());
        assert!(enrichment.set_scaling(f64::INFINITY, 1.0).is_err());

        let mismatched = enrichment.calculate::<usize, _, _>(
            &rle(&[1.0, 2.0]),
            &rle(&[1.0]),
            identical as fn(&f64, &f64) -> bool,
            rle(&[]),
        );
        assert!(mismatched.is_err());
    }
}
