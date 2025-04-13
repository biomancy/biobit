use super::results::{BatchHits, Hits};
use biobit_core_rs::{loc::Interval, num::PrimInt};

/// A builder for constructing interval tree data structures.
///
/// This trait implements the Builder pattern for interval trees, allowing for incremental
/// construction of an interval tree by adding intervals and their associated data elements.
/// Once all intervals are added, the `build` method can be called to construct the final tree
/// structure optimized for queries.
pub trait Builder {
    /// The type of interval tree that will be constructed.
    type Target: ITree;

    /// Add an interval and its corresponding element to the tree.
    fn add(
        self,
        interval: Interval<<Self::Target as ITree>::Idx>,
        element: <Self::Target as ITree>::Data,
    ) -> Self;

    /// Extend the tree from an iterator of intervals and their corresponding elements.
    fn extend(
        self,
        data: impl IntoIterator<
            Item = (
                Interval<<Self::Target as ITree>::Idx>,
                <Self::Target as ITree>::Data,
            ),
        >,
    ) -> Self;

    /// Build and return the final interval tree structure.
    fn build(self) -> Self::Target;
}

/// Represents an interval tree data structure for efficiently finding intervals that overlap with a query.
///
/// An interval tree organizes intervals to allow for fast overlap queries. This trait defines
/// the core operations that all interval tree implementations must provide.
pub trait ITree {
    /// The type used for interval coordinates (start and end positions).
    type Idx: PrimInt;

    /// The type of data associated with each interval in the tree.
    type Data;

    /// Returns an iterator over all data values stored in the tree.
    ///
    /// The order of elements in the iterator is implementation-defined and should not be relied upon.
    fn data(&self) -> impl Iterator<Item = &Self::Data> + '_;

    /// Returns an iterator over all intervals stored in the tree.
    ///
    /// The order of intervals in the iterator is implementation-defined and should not be relied upon.
    fn intervals(&self) -> impl Iterator<Item = Interval<Self::Idx>>;

    /// Returns an iterator over all (interval, data) pairs stored in the tree.
    ///
    /// The order of pairs in the iterator is implementation-defined and should not be relied upon.
    fn records(&self) -> impl Iterator<Item = (Interval<Self::Idx>, &Self::Data)>;

    /// Finds all entries whose intervals intersect with the given query interval.
    ///
    /// Results are placed into the provided `buffer`, overwriting its previous contents.
    /// The uniqueness of interval/data pairs in results is not guaranteed unless
    /// otherwise specified by the implementation.
    ///
    /// # Arguments
    /// * `interval` - The query interval to find overlaps with
    /// * `buffer` - A mutable buffer to store the intersection results
    fn intersect_interval<'hits, 'tree: 'hits>(
        &'tree self,
        interval: &Interval<Self::Idx>,
        buffer: &mut Hits<'hits, Self::Idx, Self::Data>,
    );

    /// Batch processes multiple interval queries, finding all entries intersecting with each query interval.
    ///
    /// This method processes multiple queries at once, which may be more efficient than
    /// processing each query individually.
    ///
    /// Results are placed into the provided `buffer`, organizing results by query interval and
    /// overwriting its previous contents. The uniqueness of interval/data pairs in results for
    /// each query interval is not guaranteed unless otherwise specified by the implementation.
    ///
    /// # Arguments
    /// * `intervals` - A slice of query intervals to process
    /// * `buffer` - A buffer to store results for each query
    fn batch_intersect_intervals<'hits, 'tree: 'hits>(
        &'tree self,
        intervals: &[Interval<Self::Idx>],
        buffer: &mut BatchHits<'hits, Self::Idx, Self::Data>,
    );
}
