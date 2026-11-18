use super::{BitComplement, Bits, Codec};
use crate::Scalar;

/// Canonical two-bit DNA alphabet.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Dna {
    A = 0,
    C = 1,
    G = 2,
    T = 3,
}

impl Codec for Dna {
    type Symbol = Self;
    type Bits = Bits<2>;

    #[inline(always)]
    fn encode(symbol: &Self::Symbol) -> Scalar<Self> {
        // SAFETY: `Dna` discriminants are exactly the valid two-bit codewords.
        unsafe { Scalar::new_unchecked(*symbol as u8) }
    }

    #[inline(always)]
    fn decode(scalar: Scalar<Self>) -> Self::Symbol {
        match scalar.store() {
            0 => Self::A,
            1 => Self::C,
            2 => Self::G,
            3 => Self::T,
            _ => unreachable!("DNA scalars use two bits"),
        }
    }
}

// SAFETY: The two-bit DNA code space is fully populated, and flipping both
// bits maps A <-> T and C <-> G.
unsafe impl BitComplement for Dna {}

#[cfg(test)]
mod tests {
    use super::{Codec, Dna};
    use crate::{Len, PackedArray};

    #[test]
    fn codec_round_trips_bases() {
        for base in [Dna::A, Dna::C, Dna::G, Dna::T] {
            assert_eq!(Dna::decode(Dna::encode(&base)), base);
        }
    }

    #[test]
    fn complement_matches_watson_crick_pairs() {
        for (base, complement) in [
            (Dna::A, Dna::T),
            (Dna::C, Dna::G),
            (Dna::G, Dna::C),
            (Dna::T, Dna::A),
        ] {
            let mut array = PackedArray::<Dna, Len<1>>::from_array(&[Dna::encode(&base)]);
            array.complement();

            assert_eq!(Dna::decode(array.first()), complement);
        }
    }
}
