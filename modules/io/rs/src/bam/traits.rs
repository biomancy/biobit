use std::io;
use std::marker::PhantomData;

use ::higher_kinded_types::prelude::*;
use noodles::bam::Record;

use biobit_core_rs::num::PrimInt;
use biobit_core_rs::{loc::Contig, LendingIterator};

pub use crate::bam::adapters::{
    AlignmentSegmentAdapter, PairedEndAlignmentSegmentsAdapter, PairedEndBundler,
};
use crate::bam::{alignment_segments::AlignmentSegments, strdeductor::StrDeductor};

pub trait IndexedBAM: Send + Sync {
    type Idx: PrimInt;
    type Ctg: Contig;
    type Item: ForLt;

    /// Fetch the data from a specific region of the reference genome.
    /// * `contig` - The contig to fetch reads from.
    /// * `start` - The start position of the region.
    /// * `end` - The end position of the region.
    /// * Returns an iterator of alignment blocks in the region.
    fn fetch<'borrow>(
        &'borrow mut self,
        contig: &Self::Ctg,
        start: Self::Idx,
        end: Self::Idx,
    ) -> io::Result<
        Box<
            dyn 'borrow
                + LendingIterator<Item = For!(<'iter> = io::Result<<Self::Item as ForLt>::Of<'iter>>)>,
        >,
    >;

    fn bal(&self) -> &'static str {
        "bal"
    }

    /// Return a new data source that injects a transformation function into the data fetching process.
    fn with_transform<Func, InItem, Args, OutItem>(
        self,
        transform: Func,
        args: Args,
    ) -> Transform<Self, Func, InItem, Args, OutItem>
    where
        Self: Sized,
        Args: Sized + Clone,
    {
        Transform {
            inner: self,
            transform,
            args,
            _phantom: <_>::default(),
        }
    }

    /// Clone the data source into a boxed trait object.
    fn cloned<'borrow>(
        &'borrow self,
    ) -> Box<dyn 'borrow + Sync + IndexedBAM<Idx = Self::Idx, Ctg = Self::Ctg, Item = Self::Item>>
    where
        Self: 'borrow;
}

impl<'b, Idx: PrimInt, Ctg: Contig, Item: ForLt> IndexedBAM
    for Box<dyn 'b + Sync + IndexedBAM<Idx = Idx, Ctg = Ctg, Item = Item>>
{
    type Idx = Idx;
    type Ctg = Ctg;
    type Item = Item;

    fn fetch<'borrow>(
        &'borrow mut self,
        contig: &Self::Ctg,
        start: Self::Idx,
        end: Self::Idx,
    ) -> io::Result<
        Box<
            dyn 'borrow
                + LendingIterator<Item = For!(<'iter> = io::Result<<Self::Item as ForLt>::Of<'iter>>)>,
        >,
    > {
        (**self).fetch(contig, start, end)
    }

    fn cloned<'borrow>(
        &'borrow self,
    ) -> Box<dyn 'borrow + Sync + IndexedBAM<Idx = Self::Idx, Ctg = Self::Ctg, Item = Self::Item>>
    where
        Self: 'borrow,
    {
        (**self).cloned()
    }
}

pub trait AdaptersForIndexedBAM: IndexedBAM {
    fn boxed<'b>(
        self,
    ) -> Box<dyn 'b + Sync + IndexedBAM<Idx = Self::Idx, Ctg = Self::Ctg, Item = Self::Item>>
    where
        Self: Sized + 'b,
    {
        Box::new(self)
    }

    fn pe_bundled(
        self,
    ) -> impl IndexedBAM<
        Idx = Self::Idx,
        Ctg = Self::Ctg,
        Item = For!(<'iter> = &'iter [(Record, Record)]),
    >
    where
        Self: Sized,
        for<'iter> Self::Item: ForLt<Of<'iter> = &'iter [Record]>,
    {
        fn helper<'borrow, InItem>(
            x: Box<
                dyn 'borrow
                    + LendingIterator<Item = For!(<'iter> = io::Result<<InItem as ForLt>::Of<'iter>>)>,
            >,
            _: &'borrow (),
        ) -> io::Result<
            Box<
                dyn 'borrow
                    + LendingIterator<Item = For!(<'iter> = io::Result<&'iter [(Record, Record)]>)>,
            >,
        >
        where
            for<'iter> InItem: ForLt<Of<'iter> = &'iter [Record]>,
        {
            Ok(Box::new(PairedEndBundler::new(x)))
        }
        self.with_transform::<_, Self::Item, (), For!(<'iter> = &'iter [(Record, Record)])>(
            helper::<Self::Item>,
            (),
        )
    }

    fn se_alignment_segments<S>(
        self,
        strander: S,
    ) -> impl IndexedBAM<
        Idx = Self::Idx,
        Ctg = Self::Ctg,
        Item = For!(<'iter> = &'iter AlignmentSegments<usize>),
    >
    where
        Self: Sized,
        S: StrDeductor + Clone + Send + Sync,
        for<'iter> Self::Item: ForLt<Of<'iter> = &'iter [Record]>,
    {
        fn helper<'borrow, InItem, S>(
            x: Box<
                dyn 'borrow
                    + LendingIterator<Item = For!(<'iter> = io::Result<<InItem as ForLt>::Of<'iter>>)>,
            >,
            strander: &'borrow S,
        ) -> io::Result<
            Box<
                dyn 'borrow
                    + LendingIterator<
                        Item = For!(<'iter> = io::Result<&'iter AlignmentSegments<usize>>),
                    >,
            >,
        >
        where
            S: StrDeductor + Clone,
            for<'iter> InItem: ForLt<Of<'iter> = &'iter [Record]>,
        {
            Ok(Box::new(AlignmentSegmentAdapter::new(x, strander.clone())))
        }
        self.with_transform::<_, Self::Item, S, For!(<'iter> = &'iter AlignmentSegments<usize>)>(
            helper::<Self::Item, S>,
            strander,
        )
    }

    fn pe_alignment_segments<S>(
        self,
        strander: S,
    ) -> impl IndexedBAM<
        Idx = Self::Idx,
        Ctg = Self::Ctg,
        Item = For!(<'iter> = &'iter AlignmentSegments<usize>),
    >
    where
        Self: Sized,
        S: StrDeductor + Clone + Send + Sync,
        for<'iter> Self::Item: ForLt<Of<'iter> = &'iter [(Record, Record)]>,
    {
        fn helper<'borrow, InItem, S>(
            x: Box<
                dyn 'borrow
                    + LendingIterator<Item = For!(<'iter> = io::Result<<InItem as ForLt>::Of<'iter>>)>,
            >,
            strander: &'borrow S,
        ) -> io::Result<
            Box<
                dyn 'borrow
                    + LendingIterator<
                        Item = For!(<'iter> = io::Result<&'iter AlignmentSegments<usize>>),
                    >,
            >,
        >
        where
            S: StrDeductor + Clone,
            for<'iter> InItem: ForLt<Of<'iter> = &'iter [(Record, Record)]>,
        {
            Ok(Box::new(PairedEndAlignmentSegmentsAdapter::new(
                AlignmentSegmentAdapter::new(x, strander.clone()),
            )))
        }
        self.with_transform::<_, Self::Item, S, For!(<'iter> = &'iter AlignmentSegments<usize>)>(
            helper::<Self::Item, S>,
            strander,
        )
    }
}

impl<T: IndexedBAM> AdaptersForIndexedBAM for T {}

pub struct Transform<T, Func, InItem, Args, OutItem> {
    inner: T,
    transform: Func,
    args: Args,
    _phantom: PhantomData<fn() -> (InItem, OutItem)>,
}

impl<T, Func, InItem, Args, OutItem> IndexedBAM for Transform<T, Func, InItem, Args, OutItem>
where
    T: Send + Sync + IndexedBAM<Item = InItem>,
    InItem: ForLt,
    OutItem: ForLt,
    for<'borrow> Func: FnMut(
            Box<
                dyn 'borrow
                    + LendingIterator<Item = For!(<'iter> = io::Result<<InItem as ForLt>::Of<'iter>>)>,
            >,
            &'borrow Args,
        ) -> io::Result<
            Box<
                dyn 'borrow
                    + LendingIterator<
                        Item = For!(<'iter> = io::Result<<OutItem as ForLt>::Of<'iter>>),
                    >,
            >,
        > + Clone
        + Send
        + Sync,
    Args: Clone + Send + Sync,
{
    type Idx = T::Idx;
    type Ctg = T::Ctg;
    type Item = OutItem;

    fn fetch<'borrow>(
        &'borrow mut self,
        contig: &Self::Ctg,
        start: Self::Idx,
        end: Self::Idx,
    ) -> io::Result<
        Box<
            dyn 'borrow
                + LendingIterator<Item = For!(<'iter> = io::Result<<Self::Item as ForLt>::Of<'iter>>)>,
        >,
    > {
        let inner = self.inner.fetch(contig, start, end)?;
        (self.transform)(inner, &self.args)
    }

    fn cloned<'borrow>(
        &'borrow self,
    ) -> Box<dyn 'borrow + Sync + IndexedBAM<Idx = Self::Idx, Ctg = Self::Ctg, Item = Self::Item>>
    where
        Self: 'borrow,
    {
        Box::new(Transform {
            inner: self.inner.cloned(),
            transform: self.transform.clone(),
            args: self.args.clone(),
            _phantom: Default::default(),
        })
    }
}
