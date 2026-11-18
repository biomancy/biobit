use biobit_bitpack::{BitComplement, Bits, Codec, Scalar, scalar};

pub trait TestCodec: Codec + Sized {
    fn scalar(pos: usize) -> Scalar<Self>;

    fn scalars(len: usize) -> Vec<Scalar<Self>> {
        Self::scalars_from(0, len)
    }

    fn scalars_from(offset: usize, len: usize) -> Vec<Scalar<Self>> {
        (0..len).map(|pos| Self::scalar(offset + pos)).collect()
    }
}

/// Test-only fully populated raw codec.
///
/// The semantic symbol is exactly the generated scalar store for `Bits<BITS>`.
/// This makes every masked bit pattern valid and allows mirror operations to
/// use the same `Scalar<C>` values as the packed containers.
#[derive(Clone, Copy, Debug)]
pub struct Raw<const BITS: usize>;

// SAFETY: Every masked integer is a valid symbol and decodes to itself.
unsafe impl<const BITS: usize> Codec for Raw<BITS>
where
    Bits<BITS>: scalar::Layout,
{
    type Symbol = <Bits<BITS> as scalar::Layout>::Store;
    type Bits = Bits<BITS>;

    #[inline(always)]
    fn encode(symbol: &Self::Symbol) -> Scalar<Self> {
        Scalar::new(*symbol).expect("raw test symbols are masked before encoding")
    }

    #[inline(always)]
    fn decode(scalar: Scalar<Self>) -> Self::Symbol {
        scalar.store()
    }
}

// SAFETY: `Raw<BITS>` treats every masked codeword as valid, so bitwise
// complement within the scalar mask is a total semantic operation.
unsafe impl<const BITS: usize> BitComplement for Raw<BITS> where Bits<BITS>: scalar::Layout {}

impl<const BITS: usize> TestCodec for Raw<BITS>
where
    Bits<BITS>: scalar::Layout,
    <Bits<BITS> as scalar::Layout>::Store: TryFrom<u64>,
    u64: From<<Bits<BITS> as scalar::Layout>::Store>,
{
    fn scalar(pos: usize) -> Scalar<Self> {
        let value = (pos as u64)
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .rotate_left((BITS % 63) as u32);

        let masked = value & u64::from(<Bits<BITS> as scalar::Layout>::MASK);
        let store = <Bits<BITS> as scalar::Layout>::Store::try_from(masked)
            .ok()
            .expect("masked test scalar must fit its scalar store");

        Scalar::new(store).expect("masked test scalar must fit its codec mask")
    }
}
