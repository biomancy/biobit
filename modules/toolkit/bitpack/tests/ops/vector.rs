use biobit_bitpack::{BitComplement, PackedVec, scalar, store};

use super::{ApplyInPlace, ApplyResize, GrowOp, InPlaceOp, ShrinkOp};

impl<C> ApplyInPlace<PackedVec<C>> for InPlaceOp<C>
where
    C: BitComplement,
    Vec<u64>: store::ScalarSet<<C as scalar::Layout>::Store>
        + store::Reverse<C::Bits, <C::Bits as scalar::Layout>::Store>
        + store::Complement<<C as scalar::Layout>::Store>
        + store::Slide<<C as scalar::Layout>::Store>,
{
    fn apply(self, target: &mut PackedVec<C>) -> bool {
        match self {
            Self::Set { pos, scalar } => target.set(pos, scalar),
            Self::Reverse => {
                target.reverse();

                true
            }
            Self::Complement => {
                target.complement();

                true
            }
            Self::SlideLeft { scalar } => target.slide_left(scalar),
            Self::SlideRight { scalar } => target.slide_right(scalar),
        }
    }
}

impl<C> ApplyResize<PackedVec<C>> for GrowOp<C>
where
    C: biobit_bitpack::Codec,
    Vec<u64>: store::PushLeft<<C as scalar::Layout>::Store>
        + store::PushRight<<C as scalar::Layout>::Store>,
{
    type Output = PackedVec<C>;

    fn apply(self, mut target: PackedVec<C>) -> Option<Self::Output> {
        match self {
            Self::PushLeft { scalar } => {
                target.push_left(scalar);

                Some(target)
            }
            Self::PushRight { scalar } => {
                target.push_right(scalar);

                Some(target)
            }
        }
    }
}

impl<C> ApplyResize<PackedVec<C>> for ShrinkOp
where
    C: biobit_bitpack::Codec,
    Vec<u64>: store::DropLeft<<C as scalar::Layout>::Store>
        + store::DropRight<<C as scalar::Layout>::Store>,
{
    type Output = PackedVec<C>;

    fn apply(self, mut target: PackedVec<C>) -> Option<Self::Output> {
        match self {
            Self::DropLeft => target.drop_left().then_some(target),
            Self::DropRight => target.drop_right().then_some(target),
        }
    }
}
