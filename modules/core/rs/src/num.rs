use std::fmt::Debug;

/// T value is a number
pub trait Num: ::num::Num + Debug + Default + Clone {}
impl<T: ::num::PrimInt + Debug + Default> Num for T {}

/// T values are primitive integers
pub trait PrimInt: ::num::PrimInt + Debug + Default {}
impl<T: ::num::PrimInt + Debug + Default> PrimInt for T {}

/// T values are non-negative primitive integers
pub trait PrimUInt: PrimInt + ::num::Unsigned {}

impl<T: PrimInt + ::num::Unsigned> PrimUInt for T {}

/// T values are float numbers
pub trait Float: ::num::Float + Debug + Default {}

impl<T: ::num::Float + Debug + Default> Float for T {}
