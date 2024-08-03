use crate::pairwise;

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Type {
    Match,
    Mismatch,
    Equivalent,
}

pub trait Classifier {
    type Symbol;

    fn classify(&self, s1: &Self::Symbol, s2: &Self::Symbol) -> Type;
}

pub struct RNAComplementarity {}

impl Classifier for RNAComplementarity {
    type Symbol = u8;

    #[inline(always)]
    fn classify(&self, s1: &Self::Symbol, s2: &Self::Symbol) -> Type {
        match (*s1, *s2) {
            (b'A', b'U')
            | (b'U', b'A')
            | (b'G', b'C')
            | (b'C', b'G')
            | (b'G', b'U')
            | (b'U', b'G') => Type::Equivalent,
            _ => Type::Mismatch,
        }
    }
}

pub struct Equality {}

impl Classifier for Equality {
    type Symbol = u8;

    fn classify(&self, s1: &Self::Symbol, s2: &Self::Symbol) -> Type {
        if *s1 == *s2 {
            Type::Match
        } else {
            Type::Mismatch
        }
    }
}

impl From<Type> for pairwise::Op {
    fn from(value: Type) -> Self {
        match value {
            Type::Match => pairwise::Op::Match,
            Type::Mismatch => pairwise::Op::Mismatch,
            Type::Equivalent => pairwise::Op::Equivalent,
        }
    }
}
