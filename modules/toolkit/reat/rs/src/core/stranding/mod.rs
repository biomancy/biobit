use biobit_core_rs::loc::Strand;
use crate::core::read::AlignedRead;
pub use by_experiment_design::{DeduceStrandByDesign, StrandSpecificExperimentDesign};

mod by_experiment_design;

pub trait StrandDeducer<R: AlignedRead> {
    fn deduce(&self, record: &R) -> Strand;
}
