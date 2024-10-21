use super::Resolution;
pub use crate::result::ResolutionOutcomes;
use biobit_collections_rs::interval_tree::overlap::Elements;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;

#[derive(Clone, Debug)]
pub struct TopRanked<Ranker, Elt>
where
    Ranker: for<'a> FnMut(Vec<usize>, &'a [Elt]) -> Vec<usize>,
{
    ranks: Vec<usize>,
    ranker: Ranker,
    downscale_multimappers: bool,
    _phantom: std::marker::PhantomData<Elt>,
}

impl<Ranker, Elt> TopRanked<Ranker, Elt>
where
    Ranker: for<'a> FnMut(Vec<usize>, &'a [Elt]) -> Vec<usize>,
{
    pub fn new(ranker: Ranker, downscale_multimappers: bool) -> Self {
        Self {
            ranks: Vec::new(),
            ranker,
            downscale_multimappers,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Idx: PrimInt, Cnts: Float, Elt, Ranker> Resolution<Idx, Cnts, Elt> for TopRanked<Ranker, Elt>
where
    Elt: Send + Sync + Clone,
    Ranker: for<'a> FnMut(Vec<usize>, &'a [Elt]) -> Vec<usize> + Clone + Send + Sync,
{
    fn reset(&mut self, _elements: &[Elt]) {
        self.ranks = (self.ranker)(self.ranks.clone(), _elements);
    }

    fn resolve(
        &mut self,
        _alignment: &SegmentedAlignment<Idx>,
        overlap: &mut [Elements<Idx, usize>],
        _elements: &[Elt],
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcomes<Cnts>,
    ) {
        for overlap in overlap.iter() {
            // Select the element with the top rank
            let minrank = overlap
                .annotations()
                .map(|x| x.iter().map(|elt| (self.ranks[*elt], *elt)))
                .flatten()
                .min();
            match minrank {
                None => {
                    outcome.discarded = outcome.discarded + Cnts::one();
                }
                Some((_, ind)) => {
                    outcome.resolved = outcome.resolved + Cnts::one();
                    counts[ind] = counts[ind] + Cnts::one();
                }
            }
        }
    }
}
