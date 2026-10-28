use eyre::{Result, eyre};

use biobit_core_rs::num::PrimUInt;

use super::TryMerge;
use super::implementation::merge_impl;
use crate::rle_vec::{Identical, RleVec};

pub fn merge<'a, V, L, M, IOriginal, INew, Output>(
    first: &'a RleVec<V, L, IOriginal>,
    second: &'a RleVec<V, L, IOriginal>,
) -> MergeSetup<'a, V, L, M, IOriginal, INew, Output>
where
    L: PrimUInt,
    M: TryMerge<V, Output>,
    IOriginal: Identical<V>,
    INew: Identical<Output>,
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
    M: TryMerge<V, Output>,
    IOriginal: Identical<V>,
    INew: Identical<Output>,
    Output = V,
> {
    first: &'a RleVec<V, L, IOriginal>,
    second: &'a RleVec<V, L, IOriginal>,
    write_to: Option<(Vec<Output>, Vec<L>)>,
    identical: Option<INew>,
    merge: Option<M>,
}

impl<
    'a,
    V,
    Output,
    L: PrimUInt,
    M: TryMerge<V, Output>,
    IOriginal: Identical<V>,
    INew: Identical<Output>,
> MergeSetup<'a, V, L, M, IOriginal, INew, Output>
{
    pub fn save_to(mut self, buffer: impl Into<(Vec<Output>, Vec<L>)>) -> Self {
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

    pub fn build(mut self) -> Result<Merge<'a, V, L, M, IOriginal, INew, Output>> {
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

impl<'a, V, Output, L, E, Single, Both, IOriginal, INew>
    MergeSetup<'a, V, L, (Single, Both), IOriginal, INew, Output>
where
    L: PrimUInt,
    Single: FnMut(&V) -> Result<Output, E>,
    Both: FnMut(&V, &V) -> Result<Output, E>,
    IOriginal: Identical<V>,
    INew: Identical<Output>,
{
    pub fn with_merge_fns(mut self, single: Single, both: Both) -> Self {
        self.merge = Some((single, both));
        self
    }
}

pub struct Merge<
    'a,
    V,
    L: PrimUInt,
    M: TryMerge<V, Output>,
    IOriginal: Identical<V>,
    INew: Identical<Output>,
    Output = V,
> {
    first: &'a RleVec<V, L, IOriginal>,
    second: &'a RleVec<V, L, IOriginal>,
    write_to: RleVec<Output, L, INew>,
    merge: M,
}

impl<
    V,
    Output,
    L: PrimUInt,
    M: TryMerge<V, Output>,
    IOriginal: Identical<V>,
    INew: Identical<Output>,
> Merge<'_, V, L, M, IOriginal, INew, Output>
{
    pub fn run(self) -> Result<RleVec<Output, L, INew>, M::Error> {
        merge_impl(self.first, self.second, self.write_to, self.merge)
    }
}
