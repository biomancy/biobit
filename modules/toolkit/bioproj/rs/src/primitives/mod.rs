//! Shared value types and invariant-preserving collections.

mod id;
mod meta;
mod non_empty;

pub use id::Id;
pub(crate) use id::define_entity_id;
pub use meta::{Meta, MetaVal};
pub use non_empty::{IsEmpty, NonEmpty};
