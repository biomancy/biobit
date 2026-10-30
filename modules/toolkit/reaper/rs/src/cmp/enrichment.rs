#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::{Dissolve, Getters};
use eyre::{Result, eyre};

use biobit_collections_rs::rle_vec;
use biobit_collections_rs::rle_vec::{Identical, RleVec};
use biobit_core_rs::num::{Float, PrimUInt};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Debug, Dissolve, Getters)]
pub struct Enrichment<Cnts: Float> {
    pseudocount: Cnts,
}

impl<Cnts: Float> Default for Enrichment<Cnts> {
    fn default() -> Self {
        Self {
            pseudocount: Cnts::one(),
        }
    }
}

impl<Cnts: Float> Enrichment<Cnts> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pseudocount added to both normalized profiles (default: one).
    pub fn set_pseudocount(&mut self, pseudocount: Cnts) -> Result<&mut Self> {
        if !pseudocount.is_finite() || pseudocount < Cnts::zero() {
            return Err(eyre!("Pseudocount must be finite and non-negative"));
        }
        self.pseudocount = pseudocount;
        Ok(self)
    }

    /// Calculates enrichment from already-normalized signal and control models.
    /// Returns zero wherever modeled signal is zero. Else returns (signal + p) / (control + p).
    pub fn calculate<Len: PrimUInt, I: Identical<Cnts>>(
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
                    } else {
                        let numerator = signal + self.pseudocount;
                        let denominator = control + self.pseudocount;
                        if denominator == Cnts::zero() {
                            return Err(eyre!("Control plus pseudocount must be positive wherever signal is positive"));
                        }
                        let enrichment = numerator / denominator;
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
    fn custom_pseudocount_and_validation() -> Result<()> {
        let mut caller = Enrichment::new();
        caller.set_pseudocount(0.5)?;
        for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(caller.set_pseudocount(invalid).is_err());
            assert_eq!(*caller.pseudocount(), 0.5);
        }
        let result = caller.calculate(
            &rle(&[1.0, 0.0]),
            &rle(&[0.0, 0.0]),
            identical as fn(&f64, &f64) -> bool,
            rle(&[]),
        )?;
        assert_eq!(dense(&result), [3.0, 0.0]);

        caller.set_pseudocount(f64::MAX)?;
        assert!(
            caller
                .calculate(
                    &rle(&[f64::MAX]),
                    &rle(&[1.0]),
                    identical as fn(&f64, &f64) -> bool,
                    rle(&[]),
                )
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn calculates_ratios_from_normalized_profiles() -> Result<()> {
        let result = Enrichment::new().set_pseudocount(0.0)?.calculate(
            &rle(&[2.0, 0.0, 6.0, 1.0]),
            &rle(&[1.0, 0.0, 3.0, 2.0]),
            identical as fn(&f64, &f64) -> bool,
            rle(&[]),
        )?;
        assert_eq!(dense(&result), vec![2.0, 0.0, 2.0, 0.5]);
        Ok(())
    }

    #[test]
    fn rejects_zero_control_without_a_pseudocount() -> Result<()> {
        let result = Enrichment::new().set_pseudocount(0.0)?.calculate(
            &rle(&[1.0]),
            &rle(&[0.0]),
            identical as fn(&f64, &f64) -> bool,
            rle(&[]),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn rejects_invalid_inputs() {
        for (signal, control) in [
            (f64::MAX * 2.0, 1.0),
            (0.0, f64::INFINITY),
            (f64::NAN, 1.0),
            (-1.0, 1.0),
        ] {
            assert!(
                Enrichment::new()
                    .calculate(
                        &rle(&[signal]),
                        &rle(&[control]),
                        identical as fn(&f64, &f64) -> bool,
                        rle(&[]),
                    )
                    .is_err()
            );
        }

        let mismatched = Enrichment::new().calculate(
            &rle(&[1.0, 2.0]),
            &rle(&[1.0]),
            identical as fn(&f64, &f64) -> bool,
            rle(&[]),
        );
        assert!(mismatched.is_err());
    }
}
