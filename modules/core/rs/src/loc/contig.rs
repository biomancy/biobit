use std::fmt::Debug;
use std::hash::Hash;

/// Contig is an object that refers to an actual assembly contig. Depending on the context, can be an encoded by a string, a number, etc.
pub trait Contig:
    Hash + PartialEq + Eq + PartialOrd + Ord + Clone + Default + Debug + Send + Sync
{
}

impl<T: Hash + PartialEq + Eq + PartialOrd + Ord + Clone + Default + Debug + Send + Sync> Contig
    for T
{
}
