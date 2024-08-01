use num::traits::{NumAssign, PrimInt};

pub mod inv;

pub trait Coordinate: PrimInt + NumAssign {}

impl<T: PrimInt + NumAssign> Coordinate for T {}

