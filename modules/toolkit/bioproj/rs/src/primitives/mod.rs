//! Shared types and collections used by the project model.

mod entity_family;
mod file_path;
mod layout;
mod mates_orientation;
mod meta;
mod non_empty;
mod ordered_map;
mod ordered_set;
mod strandedness;
mod text;
mod untyped_id;

pub(crate) use crate::lookup::{
    Sealed as SealedLookup, impl_checked_lookup, impl_direct_lookup, impl_variant_lookup,
};
pub(crate) use entity_family::define_entity_family;
pub use file_path::{FilePath, Root};
pub use layout::Layout;
pub use mates_orientation::MatesOrientation;
pub use meta::{Meta, MetaVal};
pub use non_empty::{IsEmpty, NonEmpty};
pub use ordered_map::OrderedMap;
pub use ordered_set::OrderedSet;
pub use strandedness::Strandedness;
pub use text::Text;
pub use untyped_id::UntypedId;
pub(crate) use untyped_id::define_entity_id;
