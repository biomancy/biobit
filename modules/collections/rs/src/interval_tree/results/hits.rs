use biobit_core_rs::loc::Interval;
use biobit_core_rs::num::PrimInt;
use derive_getters::Dissolve;
use std::fmt::Debug;

/// Stores the results of a single interval tree query.
///
/// Contains intervals and references (`&'tree T`) to the associated data found
/// in the tree that overlap the query region. The lifetime parameter `'tree`
/// ensures that these references do not outlive the `IntervalTree` they
/// originate from.
///
/// This structure is designed for efficient buffer reuse in two ways:
/// 1.  **Within the same lifetime scope:** Use the [`clear`](#method.clear)
///     method before passing the buffer mutably to subsequent intersection methods.
///     This reuses the internal allocations without deallocating/reallocating.
/// 2.  **Across different lifetime scopes:** Use the [`recycle`](#method.recycle)
///     method. This consumes the current `Hits` object and returns a new, empty
///     `Hits` object that reuses the internal allocations but can be associated
///     with a new lifetime `'new_tree`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Dissolve)]
pub struct Hits<'tree, Idx: PrimInt, T: ?Sized> {
    /// The intervals found in the tree corresponding to the hits.
    intervals: Vec<Interval<Idx>>,
    /// References to the data associated with the found intervals.
    data: Vec<&'tree T>,
}

// Manual Clone implementation: Required because derive(Clone) would incorrectly
// require T: Clone. We only need to clone the Interval<Idx> and the reference &'tree T.
impl<'tree, Idx: PrimInt + Clone, T: ?Sized> Clone for Hits<'tree, Idx, T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            // Interval<Idx> must be Clone
            intervals: self.intervals.clone(),
            // Vec<&'tree T> clones the references ('copy' effectively), which is cheap
            data: self.data.clone(),
        }
    }
}

impl<'tree, Idx: PrimInt, T: ?Sized> Default for Hits<'tree, Idx, T> {
    /// Creates a new, empty `Hits` collection with no pre-allocated capacity.
    #[inline]
    fn default() -> Self {
        Self {
            intervals: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl<'tree, Idx: PrimInt, T: ?Sized> Hits<'tree, Idx, T> {
    /// Creates a new, empty `Hits` collection. Alias for [`Default::default`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new, empty `Hits` collection with pre-allocated capacity.
    ///
    /// # Arguments
    /// * `capacity` - The number of hits to allocate space for.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            intervals: Vec::with_capacity(capacity),
            data: Vec::with_capacity(capacity),
        }
    }

    /// Returns an iterator yielding pairs of `(&Interval<Idx>, &'tree T)` – interval tree records
    /// that overlapped the query.
    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&Interval<Idx>, &'tree T)> {
        self.intervals.iter().zip(self.data.iter().cloned())
    }

    /// Returns all intervals that overlapped the query.
    #[inline]
    pub fn intervals(&self) -> &[Interval<Idx>] {
        &self.intervals
    }

    /// Returns all data references associated with intervals that overlapped the query.
    #[inline]
    pub fn data(&self) -> &[&'tree T] {
        &self.data
    }

    /// Adds a hit result to the collection.
    /// Used internally by interval tree query methods.
    ///
    /// # Arguments
    /// * `interval` - The interval found in the tree.
    /// * `data` - A reference to the associated data in the tree.
    #[inline]
    pub fn push(&mut self, interval: Interval<Idx>, data: &'tree T) {
        self.intervals.push(interval);
        self.data.push(data);
    }

    /// Adds multiple hits to the collection.
    /// Used internally by interval tree query methods.
    ///
    /// # Arguments
    /// * `hits` – Interleaved data and intervals found in the tree.
    #[inline]
    pub fn extend(&mut self, hits: impl IntoIterator<Item = (Interval<Idx>, &'tree T)>) {
        for (it, data) in hits {
            self.intervals.push(it);
            self.data.push(data);
        }
    }

    /// Returns `true` if the collection contains no hits.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    /// Returns the number of hits in the collection.
    #[inline]
    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    /// Clears the collection by removing all hits while retaining the allocated capacity.
    ///
    /// This method reuses the internal buffer for subsequent queries within the same lifetime scope,
    /// and is typically called internally by interval trees before processing a new query.
    #[inline]
    pub fn clear(&mut self) {
        self.intervals.clear();
        self.data.clear();
    }

    /// Consumes this `Hits` object and returns a new, empty `Hits` object,
    /// reusing the allocated memory capacity of the original.
    ///
    /// The returned `Hits` object may be associated with a different lifetime scope (`'new_tree`).
    /// The compiler infers `'new_tree` based on the context in which the new object is used.
    #[inline]
    pub fn recycle<'new_tree>(mut self) -> Hits<'new_tree, Idx, T> {
        self.clear();
        let (intervals, data) = self.dissolve();
        let data: Vec<&'new_tree T> = data.into_iter().map(|_| unreachable!()).collect();

        Hits { intervals, data }
    }
}

/// Stores the results of multiple interval tree queries (a batch).
///
/// Results for individual queries are accessible via [`intervals`](#method.intervals) and
/// [`data`](#method.data), or by iterators [`intervals_iter`](#method.intervals_iter),
/// [`data_iter`](#method.data_iter), or [`iter`](#method.iter).
///
/// This structure supports buffer reuse similar to [`Hits`]:
/// 1.  **Within the same lifetime scope:** Use [`clear`](#method.clear).
/// 2.  **Across different lifetime scopes:** Use [`recycle`](#method.recycle).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Dissolve)]
pub struct BatchHits<'tree, Idx: PrimInt, T: ?Sized> {
    // Flattened vector of all intervals found across all queries in the batch.
    intervals: Vec<Interval<Idx>>,
    // Flattened vector of references to data associated with the found intervals.
    data: Vec<&'tree T>,
    // Stores boundaries for each query in the batch. All intervals and data records in the
    // index[i]..index[i+1] range belong to the i-th query.
    index: Vec<usize>,
}

// Manual Clone implementation to circumvent the derive requirement for T: Clone
impl<'tree, Idx: PrimInt + Clone, T: ?Sized> Clone for BatchHits<'tree, Idx, T> {
    fn clone(&self) -> Self {
        Self {
            intervals: self.intervals.clone(),
            data: self.data.clone(),
            index: self.index.clone(),
        }
    }
}

impl<'tree, Idx: PrimInt, T: ?Sized> Default for BatchHits<'tree, Idx, T> {
    /// Creates a new, empty `BatchHits` collection.
    #[inline]
    fn default() -> Self {
        Self {
            intervals: Vec::new(),
            data: Vec::new(),
            index: vec![0],
        }
    }
}

impl<'tree, Idx: PrimInt, T: ?Sized> BatchHits<'tree, Idx, T> {
    /// Creates a new, empty `BatchHits` collection. Alias for [`Default::default`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new, empty `BatchHits` collection with pre-allocated capacity.
    ///
    /// # Arguments
    /// * `for_queries` - The expected number of queries.
    /// * `for_hits` - An estimate of the total number of hits across all queries.
    #[inline]
    pub fn with_capacity(for_queries: usize, for_hits: usize) -> Self {
        let mut index = Vec::with_capacity(for_queries + 1);
        index.push(0);

        Self {
            intervals: Vec::with_capacity(for_hits),
            data: Vec::with_capacity(for_hits),
            index,
        }
    }

    /// Returns a helper object to add hits for the *next* query in the batch.
    ///
    /// The returned `AddQueryHitsToBatch` object *must* have its `finish()` method
    /// called when all hits for the current query have been added. Forgetting
    /// to call `finish()` will result in a panic in debug builds when the helper
    /// object is dropped.
    #[inline]
    #[must_use = "Must call finish() on the result to finalize hits for the query"]
    pub fn add_hits<'query>(&'query mut self) -> AddQueryHitsToBatch<'query, 'tree, Idx, T>
    where
        'tree: 'query,
    {
        AddQueryHitsToBatch::new(self)
    }

    /// Returns all intervals overlapping the `i`-th query in batch or None if the batch has less
    /// than `i` queries.
    pub fn intervals(&self, i: usize) -> Option<&[Interval<Idx>]> {
        if i < self.index.len() - 1 {
            Some(&self.intervals[self.index[i]..self.index[i + 1]])
        } else {
            None
        }
    }

    /// Returns iterator over all intervals overlapping each query in the batch.
    pub fn intervals_iter(&self) -> impl ExactSizeIterator<Item = &[Interval<Idx>]> {
        (0..self.index.len() - 1)
            .into_iter()
            .map(|i| &self.intervals[self.index[i]..self.index[i + 1]])
    }

    /// Returns all data references associated with intervals overlapping the `i`-th query in batch
    /// or None if the batch has less than `i` queries.
    pub fn data(&self, i: usize) -> Option<&[&'tree T]> {
        if i < self.index.len() - 1 {
            Some(&self.data[self.index[i]..self.index[i + 1]])
        } else {
            None
        }
    }

    /// Returns iterator over all data references associated with intervals overlapping each query
    /// in the batch.
    pub fn data_iter(&self) -> impl ExactSizeIterator<Item = &[&'tree T]> {
        (0..self.index.len() - 1)
            .into_iter()
            .map(|i| &self.data[self.index[i]..self.index[i + 1]])
    }

    /// Returns an iterator over the results (intervals and data) for each query.
    /// Yields tuples `(&[Interval<Idx>], &[&'tree T])`, one tuple per query.
    /// Empty slices are yielded for queries with zero hits.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&[Interval<Idx>], &[&'tree T])> {
        (0..self.index.len() - 1).into_iter().map(|i| {
            (
                &self.intervals[self.index[i]..self.index[i + 1]],
                &self.data[self.index[i]..self.index[i + 1]],
            )
        })
    }

    /// Returns `true` if no query results have been added to the batch.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    /// Returns the number of queries processed in the batch.
    #[inline]
    pub fn len(&self) -> usize {
        self.index.len() - 1
    }

    /// Returns the total number of hits for the `i`-th query
    #[inline]
    pub fn hits(&self, i: usize) -> Option<usize> {
        if i < self.index.len() - 1 {
            Some(self.index[i + 1] - self.index[i])
        } else {
            None
        }
    }

    /// Returns the total number of hits across all queries in the batch.
    #[inline]
    pub fn total_hits(&self) -> usize {
        self.intervals.len()
    }

    /// Clears the collection, removing all hits and query counts,
    /// but retaining the allocated capacity.
    ///
    /// Use this for buffer reuse *within* the same lifetime scope `'tree`.
    #[inline]
    pub fn clear(&mut self) {
        self.intervals.clear();
        self.data.clear();
        self.index.clear();

        self.index.push(0);
    }

    /// Consumes this `BatchHits` object and returns a new, empty `BatchHits` object,
    /// reusing the allocated memory capacity of the original.
    ///
    /// Allows transferring the buffer's allocation across different lifetime scopes.
    /// The lifetime `'new_tree` is inferred by the compiler. Consumes `self`.
    #[inline]
    pub fn recycle<'new_tree>(mut self) -> BatchHits<'new_tree, Idx, T> {
        self.clear();
        let (intervals, data, index) = self.dissolve();
        let data = data.into_iter().map(|_| unreachable!()).collect();
        BatchHits {
            intervals,
            data,
            index,
        }
    }
}

/// A temporary handle for adding hits for a single query to a `BatchHits` buffer.
///
/// Obtained via [`BatchHits::add_hits`]. Use [`add`](#method.add) to add hits.
/// **Crucially**, call [`finish`](#method.finish) *exactly once* after adding all
/// hits for the current query. Failure to call `finish` causes a debug panic.
pub struct AddQueryHitsToBatch<'query, 'tree: 'query, Idx: PrimInt, T: ?Sized> {
    /// Number of hits added *for the current query*.
    length: usize,
    /// The buffer being filled. The lifetime `'a` ensures this handle doesn't outlive the buffer borrow.
    buffer: &'query mut BatchHits<'tree, Idx, T>,
}

impl<'query, 'tree: 'query, Idx: PrimInt, T: ?Sized> AddQueryHitsToBatch<'query, 'tree, Idx, T> {
    /// Creates a new handle linked to the buffer. (Internal).
    #[inline]
    fn new(buffer: &'query mut BatchHits<'tree, Idx, T>) -> Self {
        Self { length: 0, buffer }
    }

    /// Adds a single hit (interval and data reference) for the current query.
    #[inline]
    pub fn add(&mut self, interval: Interval<Idx>, data: &'tree T) {
        self.buffer.intervals.push(interval);
        self.buffer.data.push(data);
        self.length += 1;
    }

    /// Finalizes adding hits for the current query.
    ///
    /// Records the number of hits added (`self.length`) into the parent `BatchHits::index` vector.
    pub fn push(self) {
        drop(self);
    }
}

impl<'query, 'tree: 'query, Idx: PrimInt, T: ?Sized> Drop
    for AddQueryHitsToBatch<'query, 'tree, Idx, T>
{
    fn drop(&mut self) {
        debug_assert!(
            !self.buffer.index.is_empty(),
            "BatchHits buffer must always have at least one element - [0]."
        );
        self.buffer
            .index
            .push(self.buffer.index.last().unwrap() + self.length);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use eyre::Result;
    use itertools::Itertools;

    #[test]
    fn test_hits_new() {
        for hits in [Hits::<usize, str>::new(), Hits::with_capacity(10)] {
            assert!(hits.is_empty());
            assert_eq!(hits.len(), 0);
            assert_eq!(hits.intervals().len(), 0);
            assert_eq!(hits.data().len(), 0);
            assert_eq!(hits.iter().count(), 0);
            assert_eq!(hits, Hits::default());
        }
    }

    #[test]
    fn test_hits_push() -> Result<()> {
        let intervals = [Interval::new(1, 5)?, Interval::new(10, 15)?];
        let data = ["A", "B"];

        let mut hits = Hits::new();
        hits.push(intervals[0], data[0]);
        hits.push(intervals[1], data[1]);

        assert!(!hits.is_empty());
        assert_eq!(hits.len(), 2);

        assert_eq!(hits.intervals(), &intervals);

        assert_eq!(hits.data(), &[data[0], data[1]]);
        assert!(hits
            .data()
            .iter()
            .zip(data)
            .all(|(h, d)| std::ptr::eq(*h, d)));

        assert_eq!(
            hits.iter().collect_vec(),
            intervals.iter().zip(data.iter().cloned()).collect_vec()
        );
        Ok(())
    }

    #[test]
    fn test_hits_clear() -> Result<()> {
        let mut hits = Hits::new();
        hits.push(Interval::new(0, 10)?, "A");
        hits.push(Interval::new(12, 15)?, "B");
        hits.clear();

        assert!(hits.is_empty());
        assert_eq!(hits.len(), 0);
        assert_eq!(hits.intervals().len(), 0);
        assert_eq!(hits.data().len(), 0);
        assert_eq!(hits.iter().count(), 0);

        Ok(())
    }

    #[test]
    fn test_hits_clone() -> Result<()> {
        let data = "D";
        let mut hits = Hits::new();
        hits.push(Interval::new(1, 10)?, data);

        let clone = hits.clone();
        assert_eq!(hits, clone);
        assert!(hits
            .data
            .into_iter()
            .zip(clone.data.into_iter())
            .all(|(h, c)| std::ptr::eq(h, c)));
        Ok(())
    }

    #[test]
    fn test_batch_hits_new() {
        for batch in [
            BatchHits::<usize, str>::new(),
            BatchHits::with_capacity(12, 3),
        ] {
            assert!(batch.is_empty());
            assert_eq!(batch.len(), 0);
            assert_eq!(batch.total_hits(), 0);
            assert_eq!(batch.intervals_iter().count(), 0);
            assert_eq!(batch.data_iter().count(), 0);
            assert_eq!(batch.iter().count(), 0);
            assert_eq!(batch.intervals(0), None);
            assert_eq!(batch.data(0), None);
            assert_eq!(batch.index, vec![0]); // Initial state
            assert_eq!(batch, BatchHits::default());
        }
    }

    #[test]
    fn test_batch_hits_add() -> Result<()> {
        let mut batch = BatchHits::new();

        // Query 1
        let intervals_1 = vec![Interval::new(1, 10)?];
        let data_1 = vec!["A"];
        {
            let mut hits = batch.add_hits();
            for (it, data) in intervals_1.iter().zip(data_1.iter()) {
                hits.add(*it, *data);
            }
            hits.push()
        }

        assert_eq!(batch.len(), 1);
        assert_eq!(batch.total_hits(), 1);
        assert_eq!(batch.index, vec![0, 1]);

        assert_eq!(batch.hits(0), Some(1));
        assert_eq!(batch.hits(1), None);
        assert_eq!(batch.hits(2), None);
        assert_eq!(batch.hits(3), None);

        assert_eq!(batch.intervals(0), Some(intervals_1.as_slice()));
        assert_eq!(batch.intervals(1), None);
        assert_eq!(batch.intervals(2), None);
        assert_eq!(batch.intervals(3), None);
        assert_eq!(batch.intervals_iter().collect_vec(), vec![&intervals_1]);

        assert_eq!(batch.data(0), Some(data_1.as_slice()));
        assert_eq!(batch.data(1), None);
        assert_eq!(batch.data(2), None);
        assert_eq!(batch.data(3), None);
        assert_eq!(batch.data_iter().collect_vec(), vec![&data_1]);

        assert_eq!(
            batch.iter().collect_array::<1>(),
            Some([(intervals_1.as_slice(), data_1.as_slice())])
        );

        // Query 2
        let intervals_2 = vec![Interval::new(1, 13)?, Interval::new(2, 14)?];
        let data_2 = vec!["B", "C"];
        {
            let mut hits = batch.add_hits();
            for (it, data) in intervals_2.iter().zip(data_2.iter()) {
                hits.add(*it, *data);
            }
            hits.push()
        }

        assert_eq!(batch.len(), 2);
        assert_eq!(batch.total_hits(), 3);
        assert_eq!(batch.index, vec![0, 1, 3]);

        assert_eq!(batch.hits(0), Some(1));
        assert_eq!(batch.hits(1), Some(2));
        assert_eq!(batch.hits(2), None);
        assert_eq!(batch.hits(3), None);

        assert_eq!(batch.intervals(0), Some(intervals_1.as_slice()));
        assert_eq!(batch.intervals(1), Some(intervals_2.as_slice()));
        assert_eq!(batch.intervals(2), None);
        assert_eq!(batch.intervals(3), None);
        assert_eq!(
            batch.intervals_iter().collect_vec(),
            vec![&intervals_1, &intervals_2]
        );

        assert_eq!(batch.data(0), Some(data_1.as_slice()));
        assert_eq!(batch.data(1), Some(data_2.as_slice()));
        assert_eq!(batch.data(2), None);
        assert_eq!(batch.data(3), None);
        assert_eq!(batch.data_iter().collect_vec(), vec![&data_1, &data_2]);

        assert_eq!(
            batch.iter().collect_array::<2>(),
            Some([
                (intervals_1.as_slice(), data_1.as_slice()),
                (intervals_2.as_slice(), data_2.as_slice())
            ])
        );

        // Query 3
        let intervals_3 = vec![];
        let data_3 = vec![];
        {
            let mut hits = batch.add_hits();
            for (it, data) in intervals_3.iter().zip(data_3.iter()) {
                hits.add(*it, *data);
            }
            hits.push()
        }

        assert_eq!(batch.len(), 3);
        assert_eq!(batch.total_hits(), 3);
        assert_eq!(batch.index, vec![0, 1, 3, 3]);

        assert_eq!(batch.hits(0), Some(1));
        assert_eq!(batch.hits(1), Some(2));
        assert_eq!(batch.hits(2), Some(0));
        assert_eq!(batch.hits(3), None);

        assert_eq!(batch.intervals(0), Some(intervals_1.as_slice()));
        assert_eq!(batch.intervals(1), Some(intervals_2.as_slice()));
        assert_eq!(batch.intervals(2), Some(intervals_3.as_slice()));
        assert_eq!(batch.intervals(3), None);
        assert_eq!(
            batch.intervals_iter().collect_vec(),
            vec![&intervals_1, &intervals_2, &intervals_3]
        );

        assert_eq!(batch.data(0), Some(data_1.as_slice()));
        assert_eq!(batch.data(1), Some(data_2.as_slice()));
        assert_eq!(batch.data(2), Some(data_3.as_slice()));
        assert_eq!(batch.data(3), None);
        assert_eq!(
            batch.data_iter().collect_vec(),
            vec![&data_1, &data_2, &data_3]
        );

        assert_eq!(
            batch.iter().collect_array::<3>(),
            Some([
                (intervals_1.as_slice(), data_1.as_slice()),
                (intervals_2.as_slice(), data_2.as_slice()),
                (intervals_3.as_slice(), data_3.as_slice())
            ])
        );

        // Query 4
        let intervals_4 = vec![Interval::new(19, 20)?];
        let data_4 = vec!["K"];
        {
            let mut hits = batch.add_hits();
            for (it, data) in intervals_4.iter().zip(data_4.iter()) {
                hits.add(*it, *data);
            }
            hits.push()
        }

        assert_eq!(batch.len(), 4);
        assert_eq!(batch.hits(3), Some(1));
        assert_eq!(batch.total_hits(), 4);
        assert_eq!(batch.index, vec![0, 1, 3, 3, 4]);

        assert_eq!(batch.hits(0), Some(1));
        assert_eq!(batch.hits(1), Some(2));
        assert_eq!(batch.hits(2), Some(0));
        assert_eq!(batch.hits(3), Some(1));

        assert_eq!(batch.intervals(0), Some(intervals_1.as_slice()));
        assert_eq!(batch.intervals(1), Some(intervals_2.as_slice()));
        assert_eq!(batch.intervals(2), Some(intervals_3.as_slice()));
        assert_eq!(batch.intervals(3), Some(intervals_4.as_slice()));
        assert_eq!(
            batch.intervals_iter().collect_vec(),
            vec![&intervals_1, &intervals_2, &intervals_3, &intervals_4]
        );

        assert_eq!(batch.data(0), Some(data_1.as_slice()));
        assert_eq!(batch.data(1), Some(data_2.as_slice()));
        assert_eq!(batch.data(2), Some(data_3.as_slice()));
        assert_eq!(batch.data(3), Some(data_4.as_slice()));
        assert_eq!(
            batch.data_iter().collect_vec(),
            vec![&data_1, &data_2, &data_3, &data_4]
        );

        assert_eq!(
            batch.iter().collect_array::<4>(),
            Some([
                (intervals_1.as_slice(), data_1.as_slice()),
                (intervals_2.as_slice(), data_2.as_slice()),
                (intervals_3.as_slice(), data_3.as_slice()),
                (intervals_4.as_slice(), data_4.as_slice())
            ])
        );

        // Make sure that all pointers are pointing to the original string
        for (query, expected) in batch.data_iter().zip([data_1, data_2, data_3, data_4]) {
            for (q, e) in query.into_iter().zip(expected.into_iter()) {
                assert!(std::ptr::eq(*q, e))
            }
        }

        Ok(())
    }

    #[test]
    fn test_batch_clear() -> Result<()> {
        let mut batch = BatchHits::new();

        {
            batch.add_hits().add(Interval::new(10, 12)?, "A")
        }

        // Clear the batch
        batch.clear();

        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
        assert_eq!(batch.hits(0), None);
        assert_eq!(batch.total_hits(), 0);

        assert_eq!(batch.data(0), None);
        assert_eq!(batch.data_iter().count(), 0);

        assert_eq!(batch.intervals(0), None);
        assert_eq!(batch.intervals_iter().count(), 0);

        assert_eq!(batch.iter().count(), 0);

        Ok(())
    }

    #[test]
    fn test_batch_hits_clone() -> Result<()> {
        let data = "A";
        let mut batch1: BatchHits<usize, str> = BatchHits::new();
        {
            batch1.add_hits().add(Interval::new(0, 10)?, data);
        }

        let batch2 = batch1.clone();
        assert_eq!(batch1, batch2);

        for (b1, b2) in batch1.data.iter().zip(batch2.data.iter()) {
            assert!(std::ptr::eq(*b1, *b2))
        }
        Ok(())
    }
}
