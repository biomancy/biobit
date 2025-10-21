use crate::core::read::AlignedRead;
use biobit_core_rs::loc::Strand;
pub use by_experiment_design::{DeduceStrandByDesign, StrandSpecificExperimentDesign};

mod by_experiment_design;

pub trait StrandDeducer<R: AlignedRead> {
    fn deduce(&self, record: &R) -> Strand;
}
