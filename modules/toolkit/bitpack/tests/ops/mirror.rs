use biobit_bitpack::{BitComplement, Codec, Scalar, scalar};

use super::{ApplyInPlace, ApplyResize, GrowOp, InPlaceOp, ShrinkOp};

#[inline(always)]
fn complement_scalar<C>(scalar: Scalar<C>) -> Scalar<C>
where
    C: BitComplement,
{
    let store = (!scalar.store()) & <C as scalar::Layout>::MASK;

    unsafe { Scalar::new_unchecked(store) }
}

impl<C> ApplyInPlace<Vec<Scalar<C>>> for InPlaceOp<C>
where
    C: BitComplement,
{
    fn apply(self, target: &mut Vec<Scalar<C>>) -> bool {
        match self {
            Self::Set { pos, scalar } => {
                if pos >= target.len() {
                    return false;
                }

                target[pos] = scalar;

                true
            }
            Self::Reverse => {
                target.reverse();

                true
            }
            Self::Complement => {
                for scalar in target {
                    *scalar = complement_scalar(*scalar);
                }

                true
            }
            Self::SlideLeft { scalar } => {
                if target.is_empty() {
                    return false;
                }

                target.insert(0, scalar);
                target.pop();

                true
            }
            Self::SlideRight { scalar } => {
                if target.is_empty() {
                    return false;
                }

                target.remove(0);
                target.push(scalar);

                true
            }
        }
    }
}

impl<C> ApplyResize<Vec<Scalar<C>>> for GrowOp<C>
where
    C: Codec,
{
    type Output = Vec<Scalar<C>>;

    fn apply(self, mut target: Vec<Scalar<C>>) -> Option<Self::Output> {
        match self {
            Self::PushLeft { scalar } => {
                target.insert(0, scalar);

                Some(target)
            }
            Self::PushRight { scalar } => {
                target.push(scalar);

                Some(target)
            }
        }
    }
}

impl<C> ApplyResize<Vec<Scalar<C>>> for ShrinkOp
where
    C: Codec,
{
    type Output = Vec<Scalar<C>>;

    fn apply(self, mut target: Vec<Scalar<C>>) -> Option<Self::Output> {
        match self {
            Self::DropLeft => {
                if target.is_empty() {
                    return None;
                }

                target.remove(0);

                Some(target)
            }
            Self::DropRight => {
                target.pop()?;

                Some(target)
            }
        }
    }
}
