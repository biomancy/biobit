//! Shared value types and invariant-preserving collections.

mod untyped_id;
mod meta;
mod non_empty;
mod uri;

pub use untyped_id::UntypedId;
pub(crate) use untyped_id::define_entity_id;
pub use meta::{Meta, MetaVal};
pub use non_empty::{IsEmpty, NonEmpty};
pub use uri::Uri;
