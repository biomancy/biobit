use biobit_bitpack::{BitComplement, PackedBuffer, scalar, store};

use super::{ApplyInPlace, ApplyResize, GrowOp, InPlaceOp, ShrinkOp};

impl<C, S> ApplyInPlace<PackedBuffer<C, S>> for InPlaceOp<C>
where
    C: BitComplement,
    S: store::StaticStore
        + store::ScalarSet<<C as scalar::Layout>::Store>
        + store::Reverse<C::Bits, <C::Bits as scalar::Layout>::Store>
        + store::Complement<<C as scalar::Layout>::Store>
        + store::Slide<<C as scalar::Layout>::Store>,
{
    fn apply(self, target: &mut PackedBuffer<C, S>) -> bool {
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

impl<C, S> ApplyResize<PackedBuffer<C, S>> for GrowOp<C>
where
    C: biobit_bitpack::Codec,
    S: store::StaticStore
        + store::PushLeft<<C as scalar::Layout>::Store>
        + store::PushRight<<C as scalar::Layout>::Store>,
{
    type Output = PackedBuffer<C, S>;

    fn apply(self, mut target: PackedBuffer<C, S>) -> Option<Self::Output> {
        match self {
            Self::PushLeft { scalar } => target.push_left(scalar).then_some(target),
            Self::PushRight { scalar } => target.push_right(scalar).then_some(target),
        }
    }
}

impl<C, S> ApplyResize<PackedBuffer<C, S>> for ShrinkOp
where
    C: biobit_bitpack::Codec,
    S: store::StaticStore
        + store::DropLeft<<C as scalar::Layout>::Store>
        + store::DropRight<<C as scalar::Layout>::Store>,
{
    type Output = PackedBuffer<C, S>;

    fn apply(self, mut target: PackedBuffer<C, S>) -> Option<Self::Output> {
        match self {
            Self::DropLeft => target.drop_left().then_some(target),
            Self::DropRight => target.drop_right().then_some(target),
        }
    }
}
