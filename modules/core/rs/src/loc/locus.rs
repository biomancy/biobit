use std::fmt::Display;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use derive_getters::Dissolve;
use derive_more::Constructor;
use eyre::Report;
use impl_tools::autoimpl;

use crate::num::PrimInt;

use super::contig::Contig;
use super::interval::{Interval, IntervalOp};
use super::orientation::Orientation;

///  A locus is a physical region within a genome that has both coordinates (contig and range) and an orientation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Dissolve, Constructor)]
pub struct Locus<Ctg: Contig, Idx: PrimInt> {
    pub contig: Ctg,
    pub interval: Interval<Idx>,
    pub orientation: Orientation,
}

/// Trait for types that can be generally viewed as a genomic locus.
#[autoimpl(for < T: trait + ? Sized > & T, Box < T >, Rc < T >, Arc < T >)]
pub trait AsLocus {
    type Contig: Contig;
    type Idx: PrimInt;
    type Interval: IntervalOp<Idx = Self::Idx>;

    /// Contig of the locus-like object.
    fn contig(&self) -> &Self::Contig;

    /// Interval of the locus-like object.
    fn interval(&self) -> &Self::Interval;

    /// Orientation of the locus-like object.
    fn orientation(&self) -> Orientation;

    /// Turn the locus-like object into a basic genomic locus.
    /// TODO: This should be removed in the future in favor of a more general approach, e.g. Into<Locus>.
    fn as_locus(&self) -> Locus<Self::Contig, Self::Idx> {
        Locus {
            contig: self.contig().clone(),
            interval: self.interval().as_interval(),
            orientation: self.orientation(),
        }
    }
}

impl<Ctg: Contig, Idx: PrimInt> AsLocus for Locus<Ctg, Idx> {
    type Contig = Ctg;
    type Idx = Idx;
    type Interval = Interval<Idx>;

    fn contig(&self) -> &Self::Contig {
        &self.contig
    }
    fn interval(&self) -> &Self::Interval {
        &self.interval
    }
    fn orientation(&self) -> Orientation {
        self.orientation
    }
    fn as_locus(&self) -> Locus<Self::Contig, Self::Idx> {
        self.clone()
    }
}

impl<Ctg: Contig, Idx: PrimInt> Default for Locus<Ctg, Idx> {
    fn default() -> Self {
        Self {
            contig: Ctg::default(),
            interval: Interval::default(),
            orientation: Default::default(),
        }
    }
}

impl<Ctg: Contig, Idx: PrimInt> From<(Ctg, Interval<Idx>, Orientation)> for Locus<Ctg, Idx> {
    fn from((contig, interval, orientation): (Ctg, Interval<Idx>, Orientation)) -> Self {
        Self {
            contig,
            interval,
            orientation,
        }
    }
}

impl<Ctg: Contig, Idx: PrimInt> TryFrom<(Ctg, Range<Idx>, Orientation)> for Locus<Ctg, Idx> {
    type Error = Report;

    fn try_from(
        (contig, range, orientation): (Ctg, Range<Idx>, Orientation),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            contig,
            interval: (range.start, range.end).try_into()?,
            orientation,
        })
    }
}

impl Display for Locus<String, i64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}-{}[{}]",
            self.contig,
            self.interval.start(),
            self.interval.end(),
            self.orientation
        )
    }
}
