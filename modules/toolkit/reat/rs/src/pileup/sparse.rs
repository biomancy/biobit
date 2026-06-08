use std::borrow::Borrow;

use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::num::PrimUInt;
#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use eyre::{Result, ensure};

use super::dense::DensePileup;
use super::{Pileup, Site, SiteMut};

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SparsePileup<Idx: PrimUInt = u64, Cnts: PrimUInt = u32> {
    positions: Vec<Idx>,
    counts: Pileup<Cnts>,
}

#[allow(clippy::len_without_is_empty)]
impl<Idx: PrimUInt, Cnts: PrimUInt> SparsePileup<Idx, Cnts> {
    pub fn new(positions: Vec<Idx>, counts: Pileup<Cnts>) -> Result<Self> {
        ensure!(
            counts.len() == positions.len(),
            "pileup counts length does not match sparse positions length"
        );
        ensure!(
            !positions.is_empty(),
            "sparse pileup must contain at least one position"
        );
        ensure!(
            positions.windows(2).all(|window| window[0] < window[1]),
            "sparse positions must be sorted and unique"
        );
        ensure!(
            positions
                .last()
                .is_none_or(|position| *position < Idx::max_value()),
            "last sparse position cannot be the maximum value"
        );

        Ok(Self { positions, counts })
    }

    #[inline]
    pub fn interval(&self) -> Interval<Idx> {
        let start = self
            .positions
            .first()
            .copied()
            .expect("sparse pileup should contain at least one position");
        let end = self
            .positions
            .last()
            .copied()
            .and_then(|position| position.checked_add(&Idx::one()))
            .expect("sparse positions should permit a half-open interval");
        Interval::new(start, end).expect("sparse positions should be sorted and unique")
    }

    #[inline]
    pub fn positions(&self) -> &[Idx] {
        &self.positions
    }

    #[inline]
    pub fn counts(&self) -> &Pileup<Cnts> {
        &self.counts
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Idx, Site<'_, Cnts>)> + '_ {
        self.positions.iter().copied().zip(self.counts.iter())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Idx, SiteMut<'_, Cnts>)> + '_ {
        self.positions.iter().copied().zip(self.counts.iter_mut())
    }
}

impl<Idx, Cnts> SparsePileup<Idx, Cnts>
where
    Idx: PrimUInt,
    Cnts: PrimUInt,
{
    pub fn from_dense(
        dense: &DensePileup<Idx, Cnts>,
        offsets: impl IntoIterator<Item = usize>,
    ) -> Result<Self> {
        let offsets = offsets.into_iter().collect::<Vec<_>>();
        ensure!(
            !offsets.is_empty(),
            "Selected offsets must contain at least one position"
        );
        ensure!(
            offsets.windows(2).all(|window| window[0] < window[1]),
            "Selected offsets must be sorted and unique"
        );
        ensure!(
            offsets.iter().all(|offset| *offset < dense.len()),
            "Selected offset is out of bounds"
        );

        Ok(unsafe { Self::from_dense_unchecked(dense, offsets) })
    }

    /// # Safety
    /// `offsets` must be sorted, unique, non-empty, and each offset must be less than `dense.len()`.
    /// Violating these conditions may cause panics or undefined behavior.
    pub(crate) unsafe fn from_dense_unchecked(
        dense: &DensePileup<Idx, Cnts>,
        offsets: Vec<usize>,
    ) -> Self {
        // Offsets are local coordinates inside the dense pileup interval.
        debug_assert!(
            !offsets.is_empty(),
            "Selected offsets must contain at least one position"
        );
        debug_assert!(
            offsets.windows(2).all(|window| window[0] < window[1]),
            "Selected offsets must be sorted and unique"
        );
        debug_assert!(
            offsets.iter().all(|offset| *offset < dense.len()),
            "Selected offset is out of bounds"
        );

        let capacity = offsets.len();
        let mut positions = Vec::with_capacity(capacity);
        let mut a = Vec::with_capacity(capacity);
        let mut c = Vec::with_capacity(capacity);
        let mut g = Vec::with_capacity(capacity);
        let mut t = Vec::with_capacity(capacity);
        let mut n = Vec::with_capacity(capacity);
        let mut deletion = Vec::with_capacity(capacity);

        let mut dense_sites = dense.iter();
        let mut next_offset = 0;
        for offset in offsets {
            let skip = offset - next_offset;
            let (position, site) = dense_sites
                .nth(skip)
                .expect("selected offset should be in bounds");
            next_offset = offset + 1;

            positions.push(position);
            a.push(*site.a());
            c.push(*site.c());
            g.push(*site.g());
            t.push(*site.t());
            n.push(*site.n());
            deletion.push(*site.deletion());
        }

        let counts =
            Pileup::new(a, c, g, t, n, deletion).expect("selected offsets should be in bounds");
        SparsePileup { positions, counts }
    }

    pub fn from_distinct_chunks(chunks: &[impl Borrow<Self>]) -> Result<Self> {
        let mut chunk = chunks
            .first()
            .ok_or_else(|| eyre::eyre!("sparse pileup chunks should contain at least one chunk"))?
            .borrow();
        let mut length = chunk.len();
        for nxt in &chunks[1..] {
            let nxt = nxt.borrow();
            length += nxt.len();
            ensure!(
                chunk.interval().end() <= nxt.interval().start(),
                "SparsePileup chunks must be sorted and non-overlapping"
            );
            chunk = nxt;
        }

        let mut positions = Vec::with_capacity(length);
        let mut a = Vec::with_capacity(length);
        let mut c = Vec::with_capacity(length);
        let mut g = Vec::with_capacity(length);
        let mut t = Vec::with_capacity(length);
        let mut n = Vec::with_capacity(length);
        let mut deletion = Vec::with_capacity(length);

        for chunk in chunks.iter() {
            let chunk = chunk.borrow();
            positions.extend_from_slice(chunk.positions());
            a.extend_from_slice(chunk.counts.a());
            c.extend_from_slice(chunk.counts.c());
            g.extend_from_slice(chunk.counts.g());
            t.extend_from_slice(chunk.counts.t());
            n.extend_from_slice(chunk.counts.n());
            deletion.extend_from_slice(chunk.counts.deletion());
        }

        SparsePileup::new(positions, Pileup::new(a, c, g, t, n, deletion)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dense() -> Result<DensePileup<u64, u32>> {
        DensePileup::new(
            Interval::new(10, 15).unwrap(),
            Pileup::new(
                vec![1, 2, 3, 4, 5],
                vec![6, 7, 8, 9, 10],
                vec![11, 12, 13, 14, 15],
                vec![16, 17, 18, 19, 20],
                vec![21, 22, 23, 24, 25],
                vec![26, 27, 28, 29, 30],
            )?,
        )
    }

    fn sparse(positions: Vec<u64>, a: Vec<u32>) -> Result<SparsePileup<u64, u32>> {
        let len = positions.len();
        SparsePileup::new(
            positions,
            Pileup::new(
                a,
                vec![0; len],
                vec![1; len],
                vec![2; len],
                vec![3; len],
                vec![4; len],
            )?,
        )
    }

    #[test]
    fn validates_sparse_pileup() -> Result<()> {
        let sparse = SparsePileup::new(
            vec![10_u64, 12, 14],
            Pileup::<u32>::new(
                vec![1, 3, 5],
                vec![6, 8, 10],
                vec![11, 13, 15],
                vec![16, 18, 20],
                vec![21, 23, 25],
                vec![26, 28, 30],
            )?,
        )?;

        assert_eq!(sparse.interval(), Interval::new(10, 15).unwrap());
        assert_eq!(sparse.positions(), &[10, 12, 14]);
        assert_eq!(sparse.counts().a(), &[1, 3, 5]);
        assert_eq!(sparse.counts().deletion(), &[26, 28, 30]);
        Ok(())
    }

    #[test]
    fn rejects_unsorted_positions() {
        assert!(SparsePileup::new(vec![12_u64, 11], Pileup::<u32>::zeros(2),).is_err());
    }

    #[test]
    fn builds_from_dense_local_offsets() -> Result<()> {
        let dense = dense()?;
        let sparse = SparsePileup::from_dense(&dense, [0, 2, 4])?;

        assert_eq!(sparse.interval(), Interval::new(10, 15).unwrap());
        assert_eq!(sparse.positions(), &[10, 12, 14]);
        assert_eq!(sparse.counts().a(), &[1, 3, 5]);
        assert_eq!(sparse.counts().deletion(), &[26, 28, 30]);
        Ok(())
    }

    #[test]
    fn iterates_positions_and_sites() -> Result<()> {
        let sparse = SparsePileup::new(
            vec![10_u64, 12, 14],
            Pileup::<u32>::new(
                vec![1, 3, 5],
                vec![6, 8, 10],
                vec![11, 13, 15],
                vec![16, 18, 20],
                vec![21, 23, 25],
                vec![26, 28, 30],
            )?,
        )?;

        let mut sites = sparse.iter();
        assert_eq!(sites.size_hint(), (3, Some(3)));

        let (position, site) = sites.next().unwrap();
        assert_eq!(position, 10);
        assert_eq!(site.offset(), 0);
        assert_eq!(*site.a(), 1);
        assert_eq!(sites.size_hint(), (2, Some(2)));
        assert_eq!(
            sparse
                .iter()
                .map(|(position, _site)| position)
                .collect::<Vec<_>>(),
            vec![10, 12, 14]
        );
        Ok(())
    }

    #[test]
    fn rejects_empty_positions() {
        assert!(SparsePileup::new(Vec::<u64>::new(), Pileup::<u32>::zeros(0),).is_err());
    }

    #[test]
    fn iter_mutates_sites() -> Result<()> {
        let mut sparse = SparsePileup::new(vec![10_u64, 12, 14], Pileup::<u32>::zeros(3))?;

        let mut sites = sparse.iter_mut();
        assert_eq!(sites.size_hint(), (3, Some(3)));

        let (position, mut site) = sites.next().unwrap();
        assert_eq!(position, 10);
        assert_eq!(site.offset(), 0);
        *site.a_mut() = 1;
        site[crate::dna::Observed::Deletion] = 31;
        assert_eq!(sites.size_hint(), (2, Some(2)));

        for (position, mut site) in sites {
            *site.a_mut() = (position - 10) as u32;
            *site.deletion_mut() = position as u32 + 20;
        }

        assert_eq!(sparse.counts().a(), &[1, 2, 4]);
        assert_eq!(sparse.counts().deletion(), &[31, 32, 34]);
        Ok(())
    }

    #[test]
    fn from_dense_rejects_unsorted_offsets() -> Result<()> {
        let dense = dense()?;
        assert!(SparsePileup::from_dense(&dense, [2, 1]).is_err());
        Ok(())
    }

    #[test]
    fn from_dense_rejects_empty_offsets() -> Result<()> {
        let dense = dense()?;
        assert!(SparsePileup::from_dense(&dense, []).is_err());
        Ok(())
    }

    #[test]
    fn from_dense_rejects_duplicate_offsets() -> Result<()> {
        let dense = dense()?;
        assert!(SparsePileup::from_dense(&dense, [1, 1]).is_err());
        Ok(())
    }

    #[test]
    fn from_dense_rejects_out_of_bounds_offsets() -> Result<()> {
        let dense = dense()?;
        assert!(SparsePileup::from_dense(&dense, [0, 50]).is_err());
        Ok(())
    }

    #[test]
    fn from_distinct_chunks_merges_ordered_chunks() -> Result<()> {
        let merged = SparsePileup::from_distinct_chunks(&[
            sparse(vec![10, 12], vec![1, 2])?,
            sparse(vec![20], vec![3])?,
        ])?;

        assert_eq!(merged.interval(), Interval::new(10, 21).unwrap());
        assert_eq!(merged.positions(), &[10, 12, 20]);
        assert_eq!(merged.counts().a(), &[1, 2, 3]);
        assert_eq!(merged.counts().capacity(), merged.len());
        Ok(())
    }

    #[test]
    fn from_distinct_chunks_rejects_empty_chunks() {
        assert!(
            SparsePileup::<u64, u32>::from_distinct_chunks(
                Vec::<SparsePileup<u64, u32>>::new().as_slice()
            )
            .is_err()
        );
    }

    #[test]
    fn from_distinct_chunks_rejects_unsorted_chunks() -> Result<()> {
        assert!(
            SparsePileup::from_distinct_chunks(&[
                sparse(vec![20], vec![2])?,
                sparse(vec![10], vec![1])?,
            ])
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn from_distinct_chunks_rejects_overlapping_chunks() -> Result<()> {
        assert!(
            SparsePileup::from_distinct_chunks(&[
                sparse(vec![10, 14], vec![1, 2])?,
                sparse(vec![13], vec![3])?,
            ])
            .is_err()
        );
        Ok(())
    }
}
