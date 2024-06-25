/// This module contains implementation for Vectors of Runs, encoded via either their length
/// (`RleVec`), or by their end position (`RpeVec`). Access to the data is provided through
/// `view::Flat`, `view::Runs`, and `view::EndPos` views.

/// Traits to mark ds supporting multiple read-only or mutable views to the underlying data.
///
/// These traits return a simple facade, and the actual views are constructed by calling
/// facade methods. Introducing a facade might seem redundant, but it is necessary to ensure a
/// consistent way of accessing all views supported by a given collection without requiring users
/// to type out the full view type.


pub trait View {
    type Output<'a>
        where Self: 'a;

    /// Returns a facade that can be used to create read-only views of the collection.
    fn view(&self) -> Self::Output<'_>;
}

pub trait ViewMut {
    type Output<'a>
        where Self: 'a;

    /// Returns a facade that can be used to create mutable views of the collection.
    fn view_mut(&mut self) -> Self::Output<'_>;
}


pub use rle_vec::RleVec;
pub use rpe_vec::RpeVec;
pub use traits::{Identical, Length, Position};

mod traits;
mod rle_vec;
mod rpe_vec;
pub mod view;
