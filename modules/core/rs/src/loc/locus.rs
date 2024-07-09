use std::fmt::Display;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use derive_getters::Dissolve;
use impl_tools::autoimpl;

use crate::num::PrimInt;

use super::contig::Contig;
use super::orientation::Orientation;
use super::segment::{Segment, SegmentLike};

///  A locus is a physical region within a genome that has both coordinates (contig and range) and an orientation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Dissolve)]
pub struct Locus<Ctg: Contig, Idx: PrimInt> {
    pub contig: Ctg,
    pub segment: Segment<Idx>,
    pub orientation: Orientation,
}

/// Trait for types that can be generally viewed as a genomic locus.
#[autoimpl(for < T: trait + ? Sized > & T, Box < T >, Rc < T >, Arc < T >)]
pub trait LocusLike {
    type Contig: Contig;
    type Idx: PrimInt;
    type Segment: SegmentLike<Idx = Self::Idx>;

    /// Contig of the locus-like object.
    fn contig(&self) -> &Self::Contig;

    /// Segment of the locus-like object.
    fn segment(&self) -> &Self::Segment;

    /// Orientation of the locus-like object.
    fn orientation(&self) -> Orientation;

    /// Turn the locus-like object into a basic genomic locus.
    /// TODO: This should be removed in the future in favor of a more general approach, e.g. Into<Locus>.
    fn as_locus(&self) -> Locus<Self::Contig, Self::Idx>;
}

impl<Ctg: Contig, Idx: PrimInt> LocusLike for Locus<Ctg, Idx> {
    type Contig = Ctg;
    type Idx = Idx;
    type Segment = Segment<Idx>;

    fn contig(&self) -> &Self::Contig {
        &self.contig
    }
    fn segment(&self) -> &Self::Segment {
        &self.segment
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
            segment: Segment::default(),
            orientation: Default::default(),
        }
    }
}

impl<Ctg: Contig, Idx: PrimInt> From<(Ctg, Segment<Idx>, Orientation)> for Locus<Ctg, Idx> {
    fn from((contig, segment, orientation): (Ctg, Segment<Idx>, Orientation)) -> Self {
        Self {
            contig,
            segment,
            orientation,
        }
    }
}

impl<Ctg: Contig, Idx: PrimInt> TryFrom<(Ctg, Range<Idx>, Orientation)> for Locus<Ctg, Idx> {
    type Error = ();

    fn try_from(
        (contig, range, orientation): (Ctg, Range<Idx>, Orientation),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            contig,
            segment: (range.start, range.end).try_into()?,
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
            self.segment.start(),
            self.segment.end(),
            self.orientation
        )
    }
}
