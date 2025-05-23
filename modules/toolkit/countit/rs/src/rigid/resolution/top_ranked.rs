use super::Resolution;
pub use crate::result::ResolutionOutcomes;
use biobit_collections_rs::interval_tree::BatchHits;
use biobit_core_rs::num::{Float, PrimInt};
use biobit_io_rs::bam::SegmentedAlignment;
use eyre::Result;

#[derive(Clone, Debug)]
pub struct TopRanked<Ranker, Elt>
where
    Ranker: for<'a> FnMut(Vec<usize>, &'a [Elt], &'a [usize]) -> Vec<usize> + Clone + Send + Sync,
{
    ranks: Vec<usize>,
    ranker: Ranker,
    _phantom: std::marker::PhantomData<Elt>,
}

impl<Ranker, Elt> TopRanked<Ranker, Elt>
where
    Ranker: for<'a> FnMut(Vec<usize>, &'a [Elt], &'a [usize]) -> Vec<usize> + Clone + Send + Sync,
{
    pub fn new(ranker: Ranker) -> Self {
        Self {
            ranks: Vec::new(),
            ranker,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Idx: PrimInt, Cnts: Float, Elt, Ranker> Resolution<Idx, Cnts, Elt> for TopRanked<Ranker, Elt>
where
    Elt: Send + Sync + Clone,
    Ranker: for<'a> FnMut(Vec<usize>, &'a [Elt], &'a [usize]) -> Vec<usize> + Clone + Send + Sync,
{
    fn reset(&mut self, elements: &[Elt], partition: &[usize]) {
        self.ranks = (self.ranker)(self.ranks.clone(), elements, partition);
    }

    fn resolve(
        &mut self,
        alignment: &SegmentedAlignment<Idx>,
        overlap: &mut BatchHits<Idx, usize>,
        counts: &mut [Cnts],
        outcome: &mut ResolutionOutcomes<Cnts>,
    ) -> Result<()> {
        debug_assert_eq!(alignment.len(), overlap.len());

        for (_, data) in overlap.iter() {
            // Select the element with the top rank
            let minrank = data.iter().map(|x| (self.ranks[**x], **x)).min();
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
        Ok(())
    }
}
