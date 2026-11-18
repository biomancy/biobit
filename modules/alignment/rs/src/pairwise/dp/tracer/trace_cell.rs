use biobit_bitpack::{Bits, Codec, Scalar};

use crate::pairwise::Op;

/// The winning direction of a traceback cell.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum BestTrace {
    Start = 0,
    GapRow = 1,
    GapCol = 2,
    Equivalent = 3,
}

/// Winning direction and independent row/column gap-extension flags.
/// The low two bits encode the direction. Bits 2 and 3 mark row and column
/// extensions respectively, including when the other direction wins. A clear
/// extension bit covers both an opened gap and an inactive gap.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Trace {
    Start = 0,
    GapRow = 1,
    GapCol = 2,
    Equivalent = 3,
    StartExtendRow = 4,
    GapRowExtendRow = 5,
    GapColExtendRow = 6,
    EquivalentExtendRow = 7,
    StartExtendCol = 8,
    GapRowExtendCol = 9,
    GapColExtendCol = 10,
    EquivalentExtendCol = 11,
    StartExtendBoth = 12,
    GapRowExtendBoth = 13,
    GapColExtendBoth = 14,
    EquivalentExtendBoth = 15,
}

impl Trace {
    /// Constructs a trace from its four-bit encoding.
    ///
    /// # Safety
    ///
    /// Bits 4 through 7 of `bits` must be zero.
    #[inline]
    pub(super) const unsafe fn from_bits(bits: u8) -> Self {
        debug_assert!(bits <= 0b1111);
        // SAFETY: The caller guarantees a value in 0..=15, all valid discriminants.
        unsafe { std::mem::transmute::<u8, Self>(bits) }
    }

    /// Combines a two-bit direction code with the gap-extension flags.
    ///
    /// # Safety
    ///
    /// Bits 2 through 7 of `direction` must be zero.
    #[inline(always)]
    pub(in crate::pairwise::dp) const unsafe fn from_direction_bits(
        direction: u8,
        row_gap_extends: bool,
        col_gap_extends: bool,
    ) -> Self {
        debug_assert!(direction <= 0b11);
        let bits = direction | (row_gap_extends as u8) << 2 | (col_gap_extends as u8) << 3;
        // SAFETY: The direction occupies bits 0 and 1, and the flags occupy bits 2 and 3.
        unsafe { Self::from_bits(bits) }
    }

    #[inline(always)]
    pub const fn best(self) -> BestTrace {
        // SAFETY: The mask yields 0..=3, all valid discriminants of BestTrace.
        unsafe { std::mem::transmute::<u8, BestTrace>((self as u8) & 0b11) }
    }

    #[inline(always)]
    pub const fn row_gap_extends(self) -> bool {
        (self as u8) & 0b0100 != 0
    }

    #[inline(always)]
    pub const fn col_gap_extends(self) -> bool {
        (self as u8) & 0b1000 != 0
    }
}

// SAFETY: The discriminants cover all sixteen four-bit codes.
unsafe impl Codec for Trace {
    type Symbol = Self;
    type Bits = Bits<4>;

    #[inline(always)]
    fn encode(trace: &Self) -> Scalar<Self> {
        // SAFETY: Each trace discriminant fits in four bits.
        unsafe { Scalar::new_unchecked(*trace as u8) }
    }

    #[inline(always)]
    fn decode(scalar: Scalar<Self>) -> Self {
        // SAFETY: The scalar holds a valid four-bit code.
        unsafe { Self::from_bits(scalar.store()) }
    }
}

impl TryFrom<BestTrace> for Op {
    type Error = ();

    fn try_from(value: BestTrace) -> Result<Self, Self::Error> {
        match value {
            BestTrace::Start => Err(()),
            BestTrace::GapRow => Ok(Op::GapFirst),
            BestTrace::GapCol => Ok(Op::GapSecond),
            BestTrace::Equivalent => Ok(Op::Equivalent),
        }
    }
}
