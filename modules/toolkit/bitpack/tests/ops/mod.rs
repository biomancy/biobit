#![allow(dead_code, unused_imports)]

mod array;
mod buffer;
mod codec;
mod mirror;
mod ops;
mod vector;
mod view;

pub use codec::{Raw, TestCodec};
pub use ops::{Apply, ApplyInPlace, ApplyResize, GrowOp, InPlaceOp, Op, ShrinkOp};
pub use view::{PackedView, assert_matches};
