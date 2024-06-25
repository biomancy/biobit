pub use ::num::{PrimInt, Float, Unsigned};

/// A type for primitive unsigned integers
pub trait PrimUInt: PrimInt + Unsigned {}

impl<T: PrimInt + Unsigned> PrimUInt for T {}
