use crate::core::dna::{NucCounts, Nucleotide};
use crate::core::mismatches::site::SiteData;

use super::MismatchesPreFilter;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ByMismatches {
    minfreq: f32,
    // Precasted values to save on convertions
    minmismatches_f32: f32,
    mincov_f32: f32,
    minmismatches_u32: u32,
    mincov_u32: u32,
}

impl ByMismatches {
    pub fn new(minmismatches: u32, minfreq: f32, mincov: u32) -> Self {
        Self {
            minfreq,
            minmismatches_f32: minmismatches as f32,
            mincov_f32: mincov as f32,
            minmismatches_u32: minmismatches,
            mincov_u32: mincov,
        }
    }

    #[inline]
    pub fn enough_mismatches_per_site(&self, reference: Nucleotide, sequenced: &NucCounts) -> bool {
        let cov = sequenced.coverage();
        let mismatch = sequenced.mismatches(reference);
        cov >= self.mincov_u32
            && mismatch >= self.minmismatches_u32
            && mismatch as f32 / cov as f32 >= self.minfreq
    }

    #[inline]
    pub fn mincov(&self) -> u32 {
        self.mincov_u32
    }

    #[inline]
    pub fn minfreq(&self) -> f32 {
        self.minfreq
    }

    #[inline]
    pub fn minmismatches(&self) -> u32 {
        self.minmismatches_u32
    }
}

impl MismatchesPreFilter<SiteData> for ByMismatches {
    #[inline]
    fn is_ok(&self, preview: &SiteData) -> bool {
        self.enough_mismatches_per_site(preview.refnuc, &preview.sequenced)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::dna::{NucCounts, Nucleotide};

    use super::*;

    #[test]
    fn ok_site() {
        let mut reference = Nucleotide::A;
        let sequenced = NucCounts {
            A: 1,
            C: 2,
            G: 3,
            T: 4,
        };

        for (expected, minmismatches, minfreq, mincov) in [
            (false, 10, 0f32, 0),
            (true, 9, 0f32, 5),
            (true, 8, 0f32, 8),
            (true, 9, 0.85f32, 9),
            (false, 9, 0.95f32, 10),
            (true, 9, 0.85f32, 10),
            (false, 9, 0.85f32, 11),
        ] {
            let filter = ByMismatches::new(minmismatches, minfreq, mincov);
            assert_eq!(
                filter.enough_mismatches_per_site(reference, &sequenced),
                expected
            );
        }

        reference = Nucleotide::Unknown;
        for (expected, minmismatches, minfreq, mincov) in [
            (true, 10, 0f32, 0),
            (true, 9, 0f32, 0),
            (false, 11, 0f32, 0),
            (true, 10, 1f32, 0),
            (false, 11, 1f32, 0),
            (true, 10, 1f32, 10),
            (false, 10, 1f32, 11),
        ] {
            let filter = ByMismatches::new(minmismatches, minfreq, mincov);
            assert_eq!(
                filter.enough_mismatches_per_site(reference, &sequenced),
                expected
            );
        }
    }
}
