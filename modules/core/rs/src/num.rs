use std::fmt::Debug;

/// T value is a number
pub trait Num: ::num::Num + Debug + Default + Copy + Send + Sync {}
impl<T: ::num::Num + Debug + Default + Copy + Send + Sync> Num for T {}

/// T values are primitive integers
pub trait PrimInt: ::num::PrimInt + Debug + Default + Send + Sync {}
impl<T: ::num::PrimInt + Debug + Default + Send + Sync> PrimInt for T {}

/// T values are non-negative primitive integers
pub trait PrimUInt: PrimInt + ::num::Unsigned {}

impl<T: PrimInt + ::num::Unsigned> PrimUInt for T {}

/// T values are float numbers
pub trait Float: ::num::Float + Debug + Default + Send + Sync {}

impl<T: ::num::Float + Debug + Default + Send + Sync> Float for T {}
