use biobit_bitpack::{Codec, Scalar};

#[derive(Clone, Copy, Debug)]
pub enum InPlaceOp<C: Codec> {
    Set { pos: usize, scalar: Scalar<C> },
    Reverse,
    Complement,
    SlideLeft { scalar: Scalar<C> },
    SlideRight { scalar: Scalar<C> },
}

#[derive(Clone, Copy, Debug)]
pub enum GrowOp<C: Codec> {
    PushLeft { scalar: Scalar<C> },
    PushRight { scalar: Scalar<C> },
}

#[derive(Clone, Copy, Debug)]
pub enum ShrinkOp {
    DropLeft,
    DropRight,
}

#[derive(Clone, Copy, Debug)]
pub enum Op<C: Codec> {
    InPlace(InPlaceOp<C>),
    Grow(GrowOp<C>),
    Shrink(ShrinkOp),
}

impl<C: Codec> Op<C> {
    #[inline(always)]
    pub fn set(pos: usize, scalar: Scalar<C>) -> Self {
        Self::InPlace(InPlaceOp::Set { pos, scalar })
    }

    #[inline(always)]
    pub fn reverse() -> Self {
        Self::InPlace(InPlaceOp::Reverse)
    }

    #[inline(always)]
    pub fn complement() -> Self {
        Self::InPlace(InPlaceOp::Complement)
    }

    #[inline(always)]
    pub fn slide_left(scalar: Scalar<C>) -> Self {
        Self::InPlace(InPlaceOp::SlideLeft { scalar })
    }

    #[inline(always)]
    pub fn slide_right(scalar: Scalar<C>) -> Self {
        Self::InPlace(InPlaceOp::SlideRight { scalar })
    }

    #[inline(always)]
    pub fn push_left(scalar: Scalar<C>) -> Self {
        Self::Grow(GrowOp::PushLeft { scalar })
    }

    #[inline(always)]
    pub fn push_right(scalar: Scalar<C>) -> Self {
        Self::Grow(GrowOp::PushRight { scalar })
    }

    #[inline(always)]
    pub fn drop_left() -> Self {
        Self::Shrink(ShrinkOp::DropLeft)
    }

    #[inline(always)]
    pub fn drop_right() -> Self {
        Self::Shrink(ShrinkOp::DropRight)
    }
}

pub trait ApplyInPlace<T> {
    fn apply(self, target: &mut T) -> bool;
}

pub trait ApplyResize<T> {
    type Output;

    fn apply(self, target: T) -> Option<Self::Output>;
}

pub trait Apply<T> {
    type Output;

    fn apply(self, target: T) -> Option<Self::Output>;
}

impl<C, T> Apply<T> for Op<C>
where
    C: Codec,
    InPlaceOp<C>: ApplyInPlace<T>,
    GrowOp<C>: ApplyResize<T, Output = T>,
    ShrinkOp: ApplyResize<T, Output = T>,
{
    type Output = T;

    fn apply(self, mut target: T) -> Option<Self::Output> {
        match self {
            Self::InPlace(operation) => operation.apply(&mut target).then_some(target),
            Self::Grow(operation) => operation.apply(target),
            Self::Shrink(operation) => operation.apply(target),
        }
    }
}
