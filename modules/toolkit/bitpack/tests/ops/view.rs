use biobit_bitpack::{
    Codec, Length, PackedArray, PackedBuffer, PackedVec, Scalar, array, scalar, store,
};

pub trait PackedView<C: Codec> {
    fn len(&self) -> usize;

    fn get(&self, pos: usize) -> Option<Scalar<C>>;

    fn unpack(&self) -> Vec<Scalar<C>>;
}

pub fn assert_matches<C, P>(packed: &P, mirror: &[Scalar<C>])
where
    C: Codec,
    P: PackedView<C>,
{
    assert_eq!(packed.len(), mirror.len());
    assert_eq!(packed.unpack().as_slice(), mirror);

    for pos in 0..mirror.len() {
        assert_eq!(packed.get(pos), Some(mirror[pos]));
    }

    assert_eq!(packed.get(mirror.len()), None);
}

#[inline(always)]
fn zero_scalar<C>() -> Scalar<C>
where
    C: Codec,
{
    unsafe { Scalar::new_unchecked(<<C as scalar::Layout>::Store as scalar::Store>::ZERO) }
}

impl<C> PackedView<C> for Vec<Scalar<C>>
where
    C: Codec,
{
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn get(&self, pos: usize) -> Option<Scalar<C>> {
        self.as_slice().get(pos).copied()
    }

    fn unpack(&self) -> Vec<Scalar<C>> {
        self.clone()
    }
}

impl<C, L> PackedView<C> for PackedArray<C, L>
where
    C: Codec + array::Layout<L>,
    L: Length,
    <C as array::Layout<L>>::Store: store::ScalarGet<<C as scalar::Layout>::Store>
        + store::Unpack<<C as scalar::Layout>::Store>,
{
    fn len(&self) -> usize {
        L::LEN.get()
    }

    fn get(&self, pos: usize) -> Option<Scalar<C>> {
        PackedArray::get(self, pos)
    }

    fn unpack(&self) -> Vec<Scalar<C>> {
        self.to_array().as_ref().to_vec()
    }
}

impl<C, S> PackedView<C> for PackedBuffer<C, S>
where
    C: Codec,
    S: store::StaticStore
        + store::ScalarGet<<C as scalar::Layout>::Store>
        + store::Unpack<<C as scalar::Layout>::Store>,
{
    fn len(&self) -> usize {
        PackedBuffer::len(self)
    }

    fn get(&self, pos: usize) -> Option<Scalar<C>> {
        PackedBuffer::get(self, pos)
    }

    fn unpack(&self) -> Vec<Scalar<C>> {
        let mut scalars = vec![zero_scalar::<C>(); self.len()];

        assert!(self.copy_to_slice(&mut scalars));

        scalars
    }
}

impl<C> PackedView<C> for PackedVec<C>
where
    C: Codec,
    [u64]: store::ScalarGet<<C as scalar::Layout>::Store>,
    Vec<u64>: store::Unpack<<C as scalar::Layout>::Store>,
{
    fn len(&self) -> usize {
        PackedVec::len(self)
    }

    fn get(&self, pos: usize) -> Option<Scalar<C>> {
        PackedVec::get(self, pos)
    }

    fn unpack(&self) -> Vec<Scalar<C>> {
        let mut scalars = vec![zero_scalar::<C>(); self.len()];

        assert!(self.copy_to_slice(&mut scalars));

        scalars
    }
}
