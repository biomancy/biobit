use ahash::HashMap;
use derive_getters::Dissolve;
use derive_more::Constructor;
use eyre::Report;
pub use eyre::Result;
use ::higher_kinded_types::prelude::*;

use biobit_collections_rs::rle_vec;
use biobit_core_rs::loc::{PerOrientation, Segment};
use biobit_core_rs::source::Source;
use biobit_core_rs::{
    loc::Contig,
    num::{Float, PrimInt},
    source::AnyMap,
};
use biobit_io_rs::bam::SegmentedAlignment;

use super::result::{Harvest, HarvestRegion};
use super::workload::Config;

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
        assert_eq!(
            query.1,
            Idx::zero(),
            "Query start must be 0, but is {:?}",
            query.1
        );

        // 1. Calculate pileup for the signal & control sources
        let (ccnts, control, mut cntcov) = config.model.model_control(
            query.clone(),
            control,
            &mut self.sources_cache,
            self.cnts_cache.pop().unwrap_or_default(),
            self.rle_cache.pop().unwrap_or_default(),
        )?;

        let (sigcnts, signal, mut sigcov, modeled) = config.model.model_signal(
            query,
            signal,
            &mut self.sources_cache,
            self.cnts_cache.pop().unwrap_or_default(),
            self.rle_cache.pop().unwrap_or_default(),
        )?;

        // 2. Calculate the enrichment
        let enrichment = self
            .rle_cache
            .pop()
            .unwrap_or_default()
            .try_map::<_, Report>(|orientation, rle| {
                let signal = signal.get(orientation);
                let control = control.get(orientation);

                config.cmp.calculate::<Idx, u32, RleIdentical<Cnts>>(
                    signal,
                    control,
                    config.model.identical(),
                    rle,
                )
            })?;

        // 3. Call peaks
        let mut peaks = PerOrientation::default();
        let mut nms = PerOrientation::default();

        for (orientation, enrichment) in enrichment.iter() {
            let mut _peaks = config.pcalling.run(enrichment);
            let _nms = config.postfilter.run(
                orientation,
                &mut _peaks,
                sigcnts.get(orientation),
                ccnts.get(orientation),
                &config.cmp.scaling,
            )?;

            *peaks.get_mut(orientation) = _peaks;
            *nms.get_mut(orientation) = _nms;
        }

        // Return signal/control memory to the cache
        self.rle_cache.push(signal);
        self.rle_cache.push(control);
        self.rle_cache.push(enrichment);

        self.cnts_cache.push(ccnts);
        self.cnts_cache.push(sigcnts);

        // 4. Save results
        let segment = Segment::new(query.1, query.2)?;
        let mut harvest = Vec::with_capacity(3);
        for (orientation, model) in modeled.into_iter() {
            // Completely ignore regions without any signal model
            if model.is_empty() {
                continue;
            }

            harvest.push(HarvestRegion::new(
                query.0.clone(),
                orientation,
                segment.clone(),
                std::mem::take(sigcov.get_mut(orientation)),
                std::mem::take(cntcov.get_mut(orientation)),
                model,
                std::mem::take(peaks.get_mut(orientation)),
                std::mem::take(nms.get_mut(orientation)),
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

        for worker in workers {
            for ((cmpind, _), peaks) in worker.comparisons.drain() {
                result[cmpind].1.extend(peaks);
            }
        }

        result
            .into_iter()
            .map(|(tag, peaks)| Harvest::new(tag, peaks))
            .collect()
    }
}
