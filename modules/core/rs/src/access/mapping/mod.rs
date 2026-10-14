//! Owning store handles and selectors applied when borrowed.

mod dynamic;
mod mapped;
mod read;
mod selector;
#[cfg(test)]
mod tests;
mod write;

pub use dynamic::{ErasedMapping, ErasedMappingMut};
pub use mapped::{Mapped, MappedMut};
pub use read::StaticMapping;
pub use selector::{Fallible, Infallible, Selector, SelectorMut, Then};
pub use write::StaticMappingMut;
