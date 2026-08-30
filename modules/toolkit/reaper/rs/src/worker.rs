use ahash::HashMap;
use derive_getters::Dissolve;
use derive_more::Constructor;
use eyre::Report;
pub use eyre::Result;
use higher_kinded_types::prelude::*;

use super::result::{Harvest, HarvestRegion};
use super::workload::Config;
use crate::pcalling::Peak;
use biobit_collections_rs::rle_vec;
use biobit_core_rs::loc::{Interval, PerOrientation};
use biobit_core_rs::source::Source;
use biobit_core_rs::{
    loc::Contig,
    num::{Float, PrimInt},
    source::AnyMap,
};
use biobit_io_rs::bam::SegmentedAlignment;

#[derive(Debug, Default, Dissolve, Constructor)]
pub struct RleIdentical<Cnts: Float> {
    pub sensitivity: Cnts,
}

impl<Cnts: Float> rle_vec::Identical<Cnts> for RleIdentical<Cnts> {
    #[inline(always)]
    fn identical(&self, first: &Cnts, second: &Cnts) -> bool {
        (*first - *second).abs() <= self.sensitivity
    }
}

#[derive(Debug, Default, Dissolve)]
pub struct Worker<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    // (Comparison ID, Query ID) -> pcalling results
    #[allow(clippy::type_complexity)]
    comparisons: HashMap<(usize, usize), Vec<HarvestRegion<Ctg, Idx, Cnts>>>,
    // Internal caches
    rle_cache: Vec<PerOrientation<rle_vec::RleVec<Cnts, u32, RleIdentical<Cnts>>>>,
    cnts_cache: Vec<PerOrientation<Vec<Cnts>>>,
    // Cache for sources
    sources_cache: AnyMap,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float> Worker<Ctg, Idx, Cnts> {
    pub fn reset(&mut self) {
        self.comparisons.clear();
        self.comparisons.shrink_to_fit();

        self.rle_cache.clear();
        self.rle_cache.shrink_to_fit();

        self.cnts_cache.clear();
        self.cnts_cache.shrink_to_fit();

        self.sources_cache.clear();
        self.sources_cache.shrink_to_fit();
    }

    pub fn calculater<Src>(
        &mut self,
        cmpind: usize,
        workind: usize,
        query: (&Ctg, Idx, Idx),
        signal: &mut [Src],
        control: &mut [Src],
        config: &Config<Idx, Cnts>,
    ) -> Result<()>
    where
        Src: Source<
                Args = For!(<'args> = (&'args Ctg, Idx, Idx)),
                Item = For!(<'iter> = std::io::Result<&'iter mut SegmentedAlignment<Idx>>),
            >,
    {
        // 1. Calculate pileup for the signal & control sources
        let mut cntmodel = self.cnts_cache.pop().unwrap_or_default();
        let (ccnts, control, mut cntcov) = config.model.model_control(
            query,
            control,
            &mut self.sources_cache,
            self.cnts_cache.pop().unwrap_or_default(),
            &mut cntmodel,
            self.rle_cache.pop().unwrap_or_default(),
        )?;

        let (sigcnts, signal, mut sigcov, modeled) = config.model.model_signal(
            query,
            signal,
            &mut self.sources_cache,
            cntmodel,
            self.rle_cache.pop().unwrap_or_default(),
        )?;

        // 2. Calculate the enrichment
        let enrichment = self
            .rle_cache
            .pop()
            .unwrap_or_default()
            .try_map::<_, Report>(|orientation, rle| {
                let signal = &signal[orientation];
                let control = &control[orientation];

                config.cmp.calculate::<Idx, u32, RleIdentical<Cnts>>(
                    signal,
                    control,
                    config.model.identical(),
                    rle,
                )
            })?;

        // 3. Call peaks
        let mut peaks: PerOrientation<Vec<Peak<_, _>>> = PerOrientation::default();
        let mut nms = PerOrientation::default();

        for (orientation, enrichment) in enrichment.iter() {
            let mut _peaks = config.pcalling.run(enrichment);
            let mut _nms = config.postfilter.run(
                orientation,
                (query.1, query.2),
                &_peaks,
                &sigcnts[orientation],
                &ccnts[orientation],
            )?;

            for peak in &mut _peaks {
                peak.shift(query.1);
            }
            for peak in &mut _nms {
                peak.shift(query.1);
            }

            peaks[orientation] = _peaks;
            nms[orientation] = _nms;
        }

        // Return signal/control memory to the cache
        self.rle_cache.push(signal);
        self.rle_cache.push(control);
        self.rle_cache.push(enrichment);

        self.cnts_cache.push(ccnts);
        self.cnts_cache.push(sigcnts);

        // 4. Save results
        let interval = Interval::new(query.1, query.2)?;
        let mut harvest = Vec::with_capacity(3);
        for (orientation, model) in modeled.into_iter() {
            // Completely ignore regions without any signal model
            if model.is_empty() {
                continue;
            }

            harvest.push(HarvestRegion::new(
                query.0.clone(),
                orientation,
                interval,
                std::mem::take(&mut sigcov[orientation]),
                std::mem::take(&mut cntcov[orientation]),
                model,
                std::mem::take(&mut peaks[orientation]),
                std::mem::take(&mut nms[orientation]),
            ));
        }

        match self.comparisons.insert((cmpind, workind), harvest) {
            Some(_) => Err(eyre::eyre!(
                "Ripper worker was called twice with the same comparison and query indices. \
                That must not happen and indicates a bug in the code."
            )),
            None => Ok(()),
        }
    }

    pub fn collapse<'a, Tag>(
        comparisons: Vec<Tag>,
        workers: impl Iterator<Item = &'a mut Worker<Ctg, Idx, Cnts>>,
    ) -> Vec<Harvest<Ctg, Idx, Cnts, Tag>>
    where
        Ctg: 'a,
        Idx: 'a,
        Cnts: 'a,
    {
        let mut result = comparisons
            .into_iter()
            .map(|x| (x, Vec::new()))
            .collect::<Vec<_>>();

        let mut regions = Vec::new();
        for worker in workers {
            regions.extend(worker.comparisons.drain());
        }
        regions.sort_by_key(|((cmpind, workind), _)| (*cmpind, *workind));
        for ((cmpind, _), peaks) in regions {
            result[cmpind].1.extend(peaks);
        }

        result
            .into_iter()
            .map(|(tag, peaks)| Harvest::new(tag, peaks))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use biobit_collections_rs::rle_vec::Identical;
    use biobit_core_rs::loc::Orientation;

    #[test]
    fn approximate_identity_uses_sensitivity() {
        let identical = RleIdentical::new(1e-3f64);

        assert!(identical.identical(&0.0, &0.0005));
        assert!(identical.identical(&1.0, &1.0005));
        assert!(!identical.identical(&0.0, &0.002));
        assert!(!identical.identical(&1.0, &1.002));
    }

    fn region(start: usize) -> HarvestRegion<String, usize, f32> {
        HarvestRegion::new(
            "chr1".to_string(),
            Orientation::Forward,
            Interval::new(start, start + 1).unwrap(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn collapse_orders_regions_by_workload_index() {
        let mut first = Worker::<String, usize, f32>::default();
        let mut second = Worker::<String, usize, f32>::default();
        first.comparisons.insert((0, 1), vec![region(10)]);
        second.comparisons.insert((0, 0), vec![region(0)]);

        let result = Worker::collapse(vec!["comparison"], [&mut first, &mut second].into_iter());
        assert_eq!(
            *result[0].regions()[0].interval(),
            Interval::new(0, 1).unwrap()
        );
        assert_eq!(
            *result[0].regions()[1].interval(),
            Interval::new(10, 11).unwrap()
        );
    }
}
