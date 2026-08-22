//! Shared value types and invariant-preserving collections.

mod meta;
mod non_empty;
mod untyped_id;
mod uri;

pub use meta::{Meta, MetaVal};
pub use non_empty::{IsEmpty, NonEmpty};
pub use untyped_id::UntypedId;
pub(crate) use untyped_id::define_entity_id;
pub use uri::Uri;
