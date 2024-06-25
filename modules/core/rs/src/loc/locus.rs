use std::ops::Range;

use derive_getters::{Dissolve, Getters};
use derive_more::{AsMut, AsRef, Constructor, From};

use crate::num::PrimInt;

use super::contig::Contig;
use super::interval::{Interval, LikeInterval};
use super::orientation::Orientation;

///  A locus is a physical region within a genome that has both coordinates (contig and range) and an orientation.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Constructor,
    Dissolve,
    AsMut,
    AsRef,
    From,
    Getters
)]
pub struct Locus<Ctg: Contig, Idx: PrimInt> {
    pub contig: Ctg,
    #[as_mut]
    #[as_ref]
    pub interval: Interval<Idx>,
    #[as_mut]
    #[as_ref]
    pub orientation: Orientation,
}

pub trait LikeLocus
{
    type Ctg: Contig;
    type Idx: PrimInt;
    type Intvl: LikeInterval<Idx=Self::Idx>;

    fn contig(&self) -> &Self::Ctg;
    fn interval(&self) -> &Self::Intvl;
    fn orientation(&self) -> Orientation;
    fn as_locus(&self) -> Locus<Self::Ctg, Self::Idx>;
}

impl<Ctg: Contig, Idx: PrimInt> LikeLocus for Locus<Ctg, Idx> {
    type Ctg = Ctg;
    type Idx = Idx;
    type Intvl = Interval<Idx>;

    fn contig(&self) -> &Self::Ctg { &self.contig }
    fn interval(&self) -> &Self::Intvl { &self.interval }
    fn orientation(&self) -> Orientation { self.orientation }
    fn as_locus(&self) -> Locus<Self::Ctg, Self::Idx> { self.clone() }
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

impl<Ctg: Contig, Idx: PrimInt> TryFrom<(Ctg, Range<Idx>, Orientation)> for Locus<Ctg, Idx> {
    type Error = ();

    fn try_from((contig, range, orientation): (Ctg, Range<Idx>, Orientation)) -> Result<Self, Self::Error> {
        Ok(Self {
            contig,
            interval: (range.start, range.end).try_into()?,
            orientation,
        })
    }
}
