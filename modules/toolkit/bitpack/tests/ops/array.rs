use biobit_bitpack::{BitComplement, PackedArray, array, len, scalar, store};

use super::{ApplyInPlace, ApplyResize, GrowOp, InPlaceOp, ShrinkOp};

impl<C, L> ApplyInPlace<PackedArray<C, L>> for InPlaceOp<C>
where
    C: BitComplement + array::Layout<L>,
    L: len::Length,
    <C as array::Layout<L>>::Store: store::ScalarSet<<C as scalar::Layout>::Store>
        + store::Reverse<C::Bits, <C::Bits as scalar::Layout>::Store>
        + store::Complement<<C as scalar::Layout>::Store>
        + store::Slide<<C as scalar::Layout>::Store>,
{
    fn apply(self, target: &mut PackedArray<C, L>) -> bool {
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
            Self::SlideLeft { scalar } => {
                target.slide_left(scalar);

                true
            }
            Self::SlideRight { scalar } => {
                target.slide_right(scalar);

                true
            }
        }
    }
}

impl<C, L> ApplyResize<PackedArray<C, L>> for GrowOp<C>
where
    C: biobit_bitpack::Codec + array::Layout<L> + array::Layout<<L as len::Next>::Next>,
    L: len::Next,
    <C as array::Layout<L>>::Store: store::GrowLeft<
            <C as array::Layout<<L as len::Next>::Next>>::Store,
            <C as scalar::Layout>::Store,
        > + store::GrowRight<
            <C as array::Layout<<L as len::Next>::Next>>::Store,
            <C as scalar::Layout>::Store,
        >,
{
    type Output = PackedArray<C, <L as len::Next>::Next>;

    fn apply(self, target: PackedArray<C, L>) -> Option<Self::Output> {
        match self {
            Self::PushLeft { scalar } => Some(target.push_left(scalar)),
            Self::PushRight { scalar } => Some(target.push_right(scalar)),
        }
    }
}

impl<C, L> ApplyResize<PackedArray<C, L>> for ShrinkOp
where
    C: biobit_bitpack::Codec + array::Layout<L> + array::Layout<<L as len::Prev>::Prev>,
    L: len::Prev,
    <C as array::Layout<L>>::Store: store::ShrinkLeft<
            <C as array::Layout<<L as len::Prev>::Prev>>::Store,
            <C as scalar::Layout>::Store,
        > + store::ShrinkRight<
            <C as array::Layout<<L as len::Prev>::Prev>>::Store,
            <C as scalar::Layout>::Store,
        >,
{
    type Output = PackedArray<C, <L as len::Prev>::Prev>;

    fn apply(self, target: PackedArray<C, L>) -> Option<Self::Output> {
        match self {
            Self::DropLeft => Some(target.drop_left()),
            Self::DropRight => Some(target.drop_right()),
        }
    }
}
