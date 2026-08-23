//! Shared value types and invariant-preserving collections.

mod kind;
mod lookup;
mod meta;
mod non_empty;
mod untyped_id;
mod uri;

pub(crate) use kind::impl_kind;
pub use lookup::Lookup;
pub(crate) use lookup::{Sealed as SealedLookup, impl_checked_lookup, impl_direct_lookup};
pub use meta::{Meta, MetaVal};
pub use non_empty::{IsEmpty, NonEmpty};
pub use untyped_id::UntypedId;
pub(crate) use untyped_id::define_entity_id;
pub use uri::Uri;
