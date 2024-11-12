use derive_getters::Dissolve;
use derive_more::Constructor;

use biobit_core_rs::loc::Contig;
use biobit_core_rs::num::{Float, PrimInt};

use crate::cmp::Enrichment;
use crate::model::RNAPileup;
use crate::pcalling::ByCutoff;
use crate::postfilter::NMS;

#[derive(Clone, PartialEq, Debug, Dissolve, Constructor)]
pub struct Config<Idx: PrimInt, Cnts: Float> {
    pub model: RNAPileup<Cnts>,
    pub cmp: Enrichment<Cnts>,
    pub pcalling: ByCutoff<Idx, Cnts>,
    pub postfilter: NMS<Idx, Cnts>,
}

#[derive(Clone, PartialEq, Debug, Dissolve)]
pub struct Workload<Ctg: Contig, Idx: PrimInt, Cnts: Float> {
    pub regions: Vec<(Ctg, Idx, Idx, Config<Idx, Cnts>)>,
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float> Default for Workload<Ctg, Idx, Cnts> {
    fn default() -> Self {
        Self {
            regions: Vec::new(),
        }
    }
}

impl<Ctg: Contig, Idx: PrimInt, Cnts: Float> Workload<Ctg, Idx, Cnts> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_region(
        &mut self,
        contig: Ctg,
        start: Idx,
        end: Idx,
        config: Config<Idx, Cnts>,
    ) -> &mut Self {
        self.regions.push((contig, start, end, config));
        self
    }

    pub fn add_regions(
        &mut self,
        iterator: impl Iterator<Item = (Ctg, Idx, Idx, Config<Idx, Cnts>)>,
    ) -> &mut Self {
        self.regions.extend(iterator);
        self
    }
}
