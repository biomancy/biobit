//! Shared value types and invariant-preserving collections.

mod entity_family;
mod lookup;
mod meta;
mod non_empty;
mod unique_map;
mod untyped_id;
mod uri;

pub(crate) use entity_family::define_entity_family;
pub use lookup::Lookup;
pub(crate) use lookup::{
    Sealed as SealedLookup, impl_checked_lookup, impl_direct_lookup, impl_variant_lookup,
};
pub use meta::{Meta, MetaVal};
pub use non_empty::{IsEmpty, NonEmpty};
pub(crate) use unique_map::UniqueMap;
pub use untyped_id::UntypedId;
pub(crate) use untyped_id::define_entity_id;
pub use uri::Uri;
