use std::cmp::Ordering;

use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::num::PrimInt;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Bin<N: PrimInt> {
    pub bin: Interval<N>,
    pub items: Vec<Interval<N>>,
}

fn cmp<N: PrimInt>(this: &Interval<N>, other: &Interval<N>) -> Ordering {
    let from_start = this.start().cmp(&other.start());
    if !from_start.is_eq() {
        return from_start;
    }

    let len = this.end() - this.start();
    let otherlen = other.end() - other.start();

    // Intervals with larget size must go first!
    len.cmp(&otherlen).reverse()
}

fn infer_bin<N: PrimInt>(seed: &Interval<N>, binsize: N) -> Interval<N> {
    let end = seed.end().max(seed.start() + binsize);
    Interval::new(seed.start(), end).unwrap()
}

pub fn bin<N: PrimInt>(mut workloads: Vec<Interval<N>>, maxbinsize: N) -> Vec<Bin<N>> {
    if workloads.is_empty() {
        return vec![];
    }
    workloads.sort_by(cmp);

    let mut result = Vec::with_capacity(workloads.len());

    let mut iter = workloads.into_iter();
    let first = iter.next().unwrap();
    let mut bin = infer_bin(&first, maxbinsize);
    let mut buffer = vec![first];
    let mut maxend = buffer[0].end();

    for work in iter {
        // outside of the bin
        if work.end() > bin.end() {
            debug_assert!(!buffer.is_empty());
            if maxend < bin.end() {
                unsafe { bin.set_end(maxend) };
            }

            // Save current
            result.push(Bin { bin, items: buffer });
            // Start the new one
            bin = infer_bin(&work, maxbinsize);
            buffer = vec![work];
            maxend = buffer[0].end();
        } else {
            maxend = maxend.max(work.end());
            buffer.push(work)
        }
    }

    // Save results for the last bin
    if maxend < bin.end() {
        unsafe { bin.set_end(maxend) };
    }
    result.push(Bin { bin, items: buffer });

    result
}

fn split_interval<N: PrimInt>(interstart: N, len: N, binsize: N) -> Vec<Interval<N>> {
    let total_bins = (len + binsize - N::one()) / binsize;
    let total_bins = total_bins.to_usize().unwrap();

    let mut bins: Vec<Interval<N>> = Vec::with_capacity(total_bins);
    while bins.len() < total_bins {
        let start = N::from(bins.len()).unwrap() * binsize;
        let end = std::cmp::min(start + binsize, len);
        debug_assert!(
            end - start <= binsize,
            "end: {:?}, start: {:?}, binsize: {:?}",
            end,
            start,
            binsize
        );
        bins.push(Interval::new(interstart + start, interstart + end).unwrap())
    }

    debug_assert!(!bins.is_empty());
    bins
}

pub fn split<N: PrimInt>(intervals: Vec<Interval<N>>, binsize: N) -> Vec<Interval<N>> {
    intervals
        .into_iter()
        .flat_map(|x| split_interval(x.start(), x.len(), binsize))
        .collect()
}

#[cfg(test)]
mod tests {
    use eyre::Result;

    use super::*;

    fn validate(input: Vec<Interval<u64>>, expected: Vec<Bin<u64>>, binsizes: &[u64]) {
        for binsize in binsizes {
            let result = bin(input.clone(), *binsize);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn non_overlapping() -> Result<()> {
        let inter: Vec<Interval<u64>> = vec![
            (10..20).try_into()?,
            (30..40).try_into()?,
            (50..60).try_into()?,
            (70..80).try_into()?,
        ];

        let expected = vec![
            Bin {
                bin: (10..20).try_into()?,
                items: vec![inter[0].clone()],
            },
            Bin {
                bin: (30..40).try_into()?,
                items: vec![inter[1].clone()],
            },
            Bin {
                bin: (50..60).try_into()?,
                items: vec![inter[2].clone()],
            },
            Bin {
                bin: (70..80).try_into()?,
                items: vec![inter[3].clone()],
            },
        ];
        validate(inter.clone(), expected, &[1, 15, 29]);

        let expected = vec![
            Bin {
                bin: (10..40).try_into()?,
                items: inter[0..2].to_vec(),
            },
            Bin {
                bin: (50..80).try_into()?,
                items: inter[2..].to_vec(),
            },
        ];
        validate(inter.clone(), expected, &[30, 40, 49]);

        let expected = vec![Bin {
            bin: (10..80).try_into()?,
            items: inter.clone(),
        }];
        validate(inter, expected, &[70]);

        Ok(())
    }

    #[test]
    fn overlapping() -> Result<()> {
        let inter: Vec<Interval<u64>> = vec![
            (0..3).try_into()?,
            (2..5).try_into()?,
            (3..7).try_into()?,
            (4..8).try_into()?,
            (4..8).try_into()?,
        ];

        let expected = vec![
            Bin {
                bin: (0..3).try_into()?,
                items: vec![inter[0].clone()],
            },
            Bin {
                bin: (2..5).try_into()?,
                items: vec![inter[1].clone()],
            },
            Bin {
                bin: (3..7).try_into()?,
                items: vec![inter[2].clone()],
            },
            Bin {
                bin: (4..8).try_into()?,
                items: vec![inter[3].clone(), inter[4].clone()],
            },
        ];
        validate(inter.clone(), expected, &[1, 2, 4]);

        let expected = vec![
            Bin {
                bin: (0..5).try_into()?,
                items: inter[0..2].to_vec(),
            },
            Bin {
                bin: (3..8).try_into()?,
                items: inter[2..].to_vec(),
            },
        ];
        validate(inter.clone(), expected, &[5, 6]);

        let expected = vec![Bin {
            bin: (0..8).try_into()?,
            items: inter.clone(),
        }];
        validate(inter, expected, &[8, 100]);
        Ok(())
    }

    #[test]
    fn none() {
        validate(vec![], vec![], &[1, 2, 3, 4]);
    }

    #[test]
    fn single() -> Result<()> {
        let inter: Interval<u64> = (100..200).try_into()?;
        let expected = vec![Bin {
            bin: (100..200).try_into()?,
            items: vec![inter.clone()],
        }];

        validate(vec![inter], expected, &[1, 100, 500]);
        Ok(())
    }

    #[test]
    fn complex() -> Result<()> {
        // Chromosome 1
        let inter_chr1 = vec![
            (1, 3).try_into()?,
            (2, 3).try_into()?,
            (100, 110).try_into()?,
        ];
        let expected_chr1 = vec![Bin {
            bin: (1, 110).try_into()?,
            items: inter_chr1.clone(),
        }];
        validate(inter_chr1.clone(), expected_chr1, &[200, 300]);

        let expected_chr1 = vec![
            Bin {
                bin: (1, 3).try_into()?,
                items: inter_chr1[..2].to_vec(),
            },
            Bin {
                bin: (100, 110).try_into()?,
                items: inter_chr1[2..3].to_vec(),
            },
        ];
        validate(inter_chr1, expected_chr1, &[50]);

        // Chromosome 2
        let inter_chr2 = vec![
            (0, 200).try_into()?,
            (30, 50).try_into()?,
            (110, 120).try_into()?,
        ];
        let expected_chr2 = vec![Bin {
            bin: (0, 200).try_into()?,
            items: inter_chr2.clone(),
        }];
        validate(inter_chr2.clone(), expected_chr2, &[200, 300, 50]);

        // Chromosome 3
        let inter_chr3 = vec![
            (0, 10).try_into()?,
            (10, 20).try_into()?,
            (20, 30).try_into()?,
            (30, 40).try_into()?,
        ];
        let expected_chr3 = vec![Bin {
            bin: (0, 40).try_into()?,
            items: inter_chr3.clone(),
        }];
        validate(inter_chr3.clone(), expected_chr3, &[200, 300, 50]);

        // Chromosome 4
        let inter_chr4: Vec<Interval<u64>> = vec![
            (1, 50).try_into()?,
            (1, 25).try_into()?,
            (1, 13).try_into()?,
            (1, 500).try_into()?,
        ];
        let expected_chr4 = vec![Bin {
            bin: (1, 500).try_into()?,
            items: vec![
                inter_chr4[3].clone(),
                inter_chr4[0].clone(),
                inter_chr4[1].clone(),
                inter_chr4[2].clone(),
            ],
        }];
        validate(inter_chr4.clone(), expected_chr4, &[200, 300, 50]);
        Ok(())
    }

    #[test]
    fn split_interval() -> Result<()> {
        let workload = (0..284).try_into()?;
        let expected: Vec<Interval<u64>> = vec![
            (0..100).try_into()?,
            (100..200).try_into()?,
            (200..284).try_into()?,
        ];

        assert_eq!(split(vec![workload], 100), expected);

        let result = split(vec![(10..999).try_into()?], 1000);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 10..999);

        Ok(())
    }
}
