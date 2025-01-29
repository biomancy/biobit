use biobit_core_rs::num::PrimInt;

pub trait Score: PrimInt {}

impl<T: PrimInt> Score for T {}
