//! Materialized excess for NMS, with one index for locating queried intervals.
//!
//! `None` means uncovered; `Some(0)` means covered without positive excess.

use std::ops::Range;

use biobit_collections_rs::rle_vec::{Identical, RleVec, merge};
use biobit_core_rs::num::{Float, PrimUInt};
use eyre::{Result, eyre};

type ExcessRle<Cnts, Len> = RleVec<Option<Cnts>, Len, fn(&Option<Cnts>, &Option<Cnts>) -> bool>;

/// Owns one excess RLE and its cumulative run ends for binary-search ROI lookup.
pub(super) struct ExcessProfile<Cnts, Len: PrimUInt> {
    excess: ExcessRle<Cnts, Len>,
    ends: Vec<usize>,
}

impl<Cnts: Float, Len: PrimUInt> ExcessProfile<Cnts, Len> {
    /// Materializes `max(signal - control, 0)` from normalized, finite,
    /// non-negative profiles spanning exactly `length` bases.
    /// Adjacent equal results are merged, without expanding the inputs per base.
    pub(super) fn new<I: Identical<Cnts>>(
        signal: &RleVec<Cnts, Len, I>,
        control: &RleVec<Cnts, Len, I>,
        length: usize,
    ) -> Result<Self> {
        let length_error = || eyre!("NMS profiles must match the query length ({length})");
        let excess: ExcessRle<Cnts, Len> = merge(signal, control)
            .with_merge_fns(
                // Neither profile may have an unmatched tail.
                |_| Err(length_error()),
                |s: &Cnts, c: &Cnts| {
                    Ok(if *s > Cnts::zero() || *c > Cnts::zero() {
                        Some((*s - *c).max(Cnts::zero()))
                    } else {
                        None
                    })
                },
            )
            .with_identical(PartialEq::eq as fn(&Option<Cnts>, &Option<Cnts>) -> bool)
            .build()?
            .run()?;
        let mut end = 0usize;
        let ends = excess
            .runs()
            .map(|(_, width)| {
                end = end.checked_add(width.to_usize()?)?;
                Some(end)
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(length_error)?;
        // Equal input lengths can still differ from the requested query length.
        if end != length {
            return Err(length_error());
        }
        Ok(Self { excess, ends })
    }

    /// Yields covered `(span, excess)` runs clipped to a half-open local query.
    /// Covered zeros are included; uncovered spans are skipped. The query must
    /// satisfy `start <= end <= length` passed to `new`.
    pub(super) fn covered_runs(
        &self,
        query: Range<usize>,
    ) -> impl Iterator<Item = (Range<usize>, Cnts)> + '_ {
        let first = self.ends.partition_point(|&end| end <= query.start);
        let (values, _) = self.excess.as_slices();
        self.ends[first..]
            .iter()
            .zip(&values[first..])
            // Turn `(end, value)` pairs into `(span, excess)` runs.
            .scan(query.start, move |start, (&end, &value)| {
                if *start >= query.end {
                    return None;
                }
                let span = *start..end.min(query.end);
                *start = end;
                Some((span, value))
            })
            // Keep only covered spans, discarding uncovered ones.
            .filter_map(|(span, value)| value.map(|excess| (span, excess)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_equal_excess_and_preserves_coverage() -> Result<()> {
        let rle = |values: &[f64]| {
            RleVec::<_, usize, _>::builder((|a: &f64, b: &f64| a == b) as fn(&f64, &f64) -> bool)
                .with_dense_values(values)
                .unwrap()
                .build()
        };
        let profile = ExcessProfile::new(
            &rle(&[0., 0., 5., 6., 1., 0., 0.]),
            &rle(&[0., 0., 1., 2., 2., 2., 0.]),
            7,
        )?;
        assert_eq!(
            profile
                .excess
                .runs()
                .map(|(v, len)| (*v, *len))
                .collect::<Vec<_>>(),
            [(None, 2), (Some(4.), 2), (Some(0.), 2), (None, 1)]
        );
        assert_eq!(
            profile.covered_runs(3..7).collect::<Vec<_>>(),
            [(3..4, 4.), (4..6, 0.)]
        );
        Ok(())
    }
}
