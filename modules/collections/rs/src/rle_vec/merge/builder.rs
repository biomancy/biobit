use eyre::{Result, eyre};

use biobit_core_rs::num::PrimUInt;

use super::TryMerge;
use super::implementation::merge_impl;
use crate::rle_vec::{Identical, RleVec};

pub fn merge<'a, V, L, M, IOriginal, INew>(
    first: &'a RleVec<V, L, IOriginal>,
    second: &'a RleVec<V, L, IOriginal>,
) -> MergeSetup<'a, V, L, M, IOriginal, INew>
where
    L: PrimUInt,
    M: TryMerge<V>,
    IOriginal: Identical<V>,
    INew: Identical<V>,
{
    MergeSetup {
        first,
        second,
        write_to: None,
        identical: None,
        merge: None,
    }
}

pub struct MergeSetup<
    'a,
    V,
    L: PrimUInt,
    M: TryMerge<V>,
    IOriginal: Identical<V>,
    INew: Identical<V>,
> {
    first: &'a RleVec<V, L, IOriginal>,
    second: &'a RleVec<V, L, IOriginal>,
    write_to: Option<(Vec<V>, Vec<L>)>,
    identical: Option<INew>,
    merge: Option<M>,
}

impl<'a, V, L: PrimUInt, M: TryMerge<V>, IOriginal: Identical<V>, INew: Identical<V>>
    MergeSetup<'a, V, L, M, IOriginal, INew>
{
    pub fn save_to(mut self, buffer: impl Into<(Vec<V>, Vec<L>)>) -> Self {
        let mut buffer = buffer.into();
        buffer.0.clear();
        buffer.1.clear();

        self.write_to = Some(buffer);
        self
    }

    pub fn with_identical(mut self, identical: INew) -> Self {
        self.identical = Some(identical);
        self
    }

    pub fn with_merge(mut self, merge: M) -> Self {
        self.merge = Some(merge);
        self
    }

    pub fn build(mut self) -> Result<Merge<'a, V, L, M, IOriginal, INew>> {
        let merge = self
            .merge
            .take()
            .ok_or_else(|| eyre!("Merge function is unspecified in rle_vec::merge."))?;
        let identical = self
            .identical
            .take()
            .ok_or_else(|| eyre!("Identical rule is unspecified in rle_vec::merge."))?;

        let (values, lengths) = self.write_to.take().unwrap_or_default();
        let write_to = RleVec::builder(identical)
            .with_buffers(values, lengths)
            .build();

        Ok(Merge {
            first: self.first,
            second: self.second,
            write_to,
            merge,
        })
    }
}

impl<'a, V, L, E, Single, Both, IOriginal, INew>
    MergeSetup<'a, V, L, (Single, Both), IOriginal, INew>
where
    L: PrimUInt,
    Single: FnMut(&V) -> Result<V, E>,
    Both: FnMut(&V, &V) -> Result<V, E>,
    IOriginal: Identical<V>,
    INew: Identical<V>,
{
    pub fn with_merge_fns(mut self, single: Single, both: Both) -> Self {
        self.merge = Some((single, both));
        self
    }
}

pub struct Merge<'a, V, L: PrimUInt, M: TryMerge<V>, IOriginal: Identical<V>, INew: Identical<V>> {
    first: &'a RleVec<V, L, IOriginal>,
    second: &'a RleVec<V, L, IOriginal>,
    write_to: RleVec<V, L, INew>,
    merge: M,
}

impl<V, L: PrimUInt, M: TryMerge<V>, IOriginal: Identical<V>, INew: Identical<V>>
    Merge<'_, V, L, M, IOriginal, INew>
{
    pub fn run(self) -> Result<RleVec<V, L, INew>, M::Error> {
        merge_impl(self.first, self.second, self.write_to, self.merge)
    }
}
