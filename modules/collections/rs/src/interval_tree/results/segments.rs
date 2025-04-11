use super::hits::Hits;
use crate::interval_tree::BatchHits;
use ahash::{HashMap, HashMapExt};
use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::num::PrimInt;
use derive_getters::Dissolve;
use eyre::{ensure, Result};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::Hash;

// Helper enum to represent events during the sweep line algorithm
#[derive(Debug, Eq)]
enum Event<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> {
    QueryStart(Idx),
    HitStart(Idx, &'tree T),
    HitEnd(Idx, &'tree T),
    QueryEnd(Idx),
}

impl<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> Event<'tree, Idx, T> {
    fn pos(&self) -> Idx {
        match self {
            Event::QueryStart(p) => *p,
            Event::HitStart(p, _) => *p,
            Event::HitEnd(p, _) => *p,
            Event::QueryEnd(p) => *p,
        }
    }

    // Defines sorting priority for events at the same position
    fn priority(&self) -> i8 {
        match self {
            Event::HitStart(_, _) => 0,
            Event::QueryStart(_) => 1,
            Event::QueryEnd(_) => 2,
            Event::HitEnd(_, _) => 3,
        }
    }
}

impl<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> PartialEq for Event<'tree, Idx, T> {
    fn eq(&self, other: &Self) -> bool {
        self.pos() == other.pos() && self.priority() == other.priority()
    }
}
impl<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> PartialOrd for Event<'tree, Idx, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> Ord for Event<'tree, Idx, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pos()
            .cmp(&other.pos())
            .then_with(|| self.priority().cmp(&other.priority()))
    }
}

/// Internal sweep-line implementation.
/// Populates the provided `segments` and `data` vectors based on query intervals and hits.
/// Errors if the lengths of `hits` and `data` don't match, or if the query intervals are empty.
fn sweep_line_into<'a, 'tree: 'a, Idx: PrimInt + 'a, T: Eq + Hash + ?Sized>(
    query: impl Iterator<Item = &'a Interval<Idx>>,
    mut hits: impl Iterator<Item = &'a Interval<Idx>>,
    mut data: impl Iterator<Item = &'a &'tree T>,
    segments: &mut Vec<Interval<Idx>>,
    segdata: &mut Vec<HashSet<&'tree T>>,
) -> Result<()> {
    // Create a vector of events
    let mut events = Vec::with_capacity(query.size_hint().0 * 2 + hits.size_hint().0 * 2);
    for q_interval in query {
        events.push(Event::QueryStart(q_interval.start()));
        events.push(Event::QueryEnd(q_interval.end()));
    }

    ensure!(
        !events.is_empty(),
        "No query intervals provided, cannot build segments."
    );

    for (it, &hdata) in hits.by_ref().zip(data.by_ref()) {
        events.push(Event::HitStart(it.start(), hdata));
        events.push(Event::HitEnd(it.end(), hdata));
    }

    ensure!(
        hits.next().is_none() && data.next().is_none(),
        "Mismatch between number of hits and data references."
    );

    // Sort events by position and priority
    events.sort_unstable();

    // Find the first segment
    let mut evind = 0;
    let mut active_query_regions: u32 = 0;
    let mut active_state = HashMap::new();
    let mut active_set = HashSet::new();
    let mut cursor = events[evind].pos();

    while evind < events.len() {
        let event = &events[evind];
        let pos = event.pos();

        if pos != cursor && active_query_regions > 0 {
            // SAFETY: Events are sorted by position and the cursor is always less than pos
            segments.push(unsafe { Interval::new_unchecked(cursor, pos) });
            segdata.push(active_set.clone());
            break;
        }

        // State update
        match event {
            Event::QueryStart(_) => active_query_regions += 1,
            Event::HitStart(_, data) => {
                active_set.insert(*data);
                *active_state.entry(data).or_insert(0) += 1;
            }
            Event::HitEnd(_, data) => {
                // SAFETY: We know that the data is in the active state because we added it
                let x = unsafe { active_state.get_mut(data).unwrap_unchecked() };
                debug_assert!(*x > 0);
                *x -= 1;
                if *x == 0 {
                    active_set.remove(data);
                }
            }
            Event::QueryEnd(_) => active_query_regions -= 1,
        };

        cursor = pos;
        evind += 1;
    }

    // There must be at least one segment at this point.
    debug_assert!(segments.len() >= 1);
    debug_assert!(segdata.len() >= 1);

    // If there are no more events, we're done
    if evind == events.len() {
        return Ok(());
    }

    // Otherwise, we need to process the remaining events
    let mut prevind = segments.len() - 1;
    cursor = segments[prevind].end();

    // Process the remaining events
    for event in events[evind..].iter() {
        let pos = event.pos();

        if pos != cursor && active_query_regions > 0 {
            // [cursor, pos) define a new segment

            // Can we merge with the last segment?
            if segments[prevind].end() == cursor && segdata[prevind] == active_set {
                // SAFETY: Events are sorted by position and the cursor is always less than pos
                unsafe { segments[prevind].set_end(pos) };
            } else {
                // SAFETY: Events are sorted by position and the cursor is always less than pos
                segments.push(unsafe { Interval::new_unchecked(cursor, pos) });
                segdata.push(active_set.clone());
                prevind += 1;
            }
        }

        // State update
        match event {
            Event::QueryStart(_) => active_query_regions += 1,
            Event::HitStart(_, data) => {
                active_set.insert(data);
                *active_state.entry(data).or_insert(0) += 1;
            }
            Event::HitEnd(_, data) => {
                // SAFETY: We know that the data is in the active state because we added it
                let x = unsafe { active_state.get_mut(data).unwrap_unchecked() };
                debug_assert!(*x > 0);
                *x -= 1;
                if *x == 0 {
                    active_set.remove(data);
                }
            }
            Event::QueryEnd(_) => active_query_regions -= 1,
        };

        cursor = pos;
    }

    debug_assert!(active_set.is_empty());
    debug_assert!(segments.len() == segdata.len());

    Ok(())
}

/// Represents the segmentation of query intervals based on overlapping hits.
///
/// This structure calculates and stores a series of non-overlapping, sorted `Interval<Idx>`
/// segments that cover all regions specified in the original query intervals. Each segment
/// is associated with the unique set of data references (`&'tree T`) from the input `Hits`
/// whose intervals overlap that specific segment.
///
/// This is useful for identifying contiguous genomic regions (or other coordinate spaces)
/// that share the exact same set of annotations or features. For example, finding all
/// parts of an exon covered only by transcript A, parts covered by both transcript A and B,
/// and parts covered only by transcript B.
///
/// The segments are generated using a sweep-line algorithm based on the start/end points
/// of the query intervals and the hit intervals.
///
/// **Key Guarantees:**
/// - Segments cover *only* regions within the original `query` intervals passed to `build`.
/// - Segments are non-overlapping.
/// - Segments are sorted by coordinate.
/// - Segments are maximal: adjacent segments will always have different sets of associated data.
/// - The `data` associated with each segment is a `HashSet` containing references (`&'tree T`)
///   to the original data items whose intervals overlapped that segment. An empty set means
///   no hits overlapped that segment.
///
/// **Performance:**
/// - The `build` operation involves sorting events and iterating through them. Its complexity
///   is roughly O((N +M)*log(N+M)), where N is the number of query intervals and
///   M is the number of hits.
/// - Storing results involves `HashSet` clones for each unique segment generated. If the number
///   of segments is large and the sets of overlapping hits are also large, this cloning can
///   impact performance and memory usage.
///
/// **Buffer Reuse:**
/// - `clear()`: Removes all segments and data, keeping the allocated memory for reuse
///   within the same lifetime (`'tree`).
/// - `recycle()`: Consumes the object, returning a new empty one with the same allocated
///   capacity but potentially associated with different types and a new lifetime (`'new_tree`).
#[derive(Clone, PartialEq, Eq, Debug, Dissolve)]
pub struct HitSegments<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> {
    segments: Vec<Interval<Idx>>,
    data: Vec<HashSet<&'tree T>>,
}

impl<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> Default for HitSegments<'tree, Idx, T> {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> HitSegments<'tree, Idx, T> {
    /// Creates a new, empty `HitSegments` collection with no pre-allocated capacity.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates new, empty `HitSegments` with pre-allocated capacity.
    ///
    /// # Arguments
    /// * `capacity` – An estimate of the number of segments expected.
    #[inline]
    pub fn with_capacity(segments: usize) -> Self {
        Self {
            segments: Vec::with_capacity(segments),
            data: Vec::with_capacity(segments),
        }
    }

    /// Calculates the segments based on the query intervals and hits using a sweep-line approach.
    ///
    /// Overwrites any existing segments and data stored in this `HitSegments` object.
    ///
    /// # Arguments
    /// * `query` – Intervals defining the regions of interest for segmentation.
    ///             Segments will only be generated within these intervals.
    /// * `hits` – A reference to the `Hits` object containing the overlaps (intervals
    ///            and associated data references `&'tree T`) found by querying an interval tree.
    pub fn build<'a, Query>(&mut self, query: Query, hits: &'a Hits<'tree, Idx, T>) -> Result<()>
    where
        Query: IntoIterator<Item = &'a Interval<Idx>>,
        Idx: 'a,
        'tree: 'a,
    {
        self.clear();
        sweep_line_into(
            query.into_iter(),
            hits.intervals().into_iter(),
            hits.data().into_iter(),
            &mut self.segments,
            &mut self.data,
        )
    }

    /// Calculates the segments based on the query intervals and hit parts. Errs if the length
    /// of intervals and data don't match.
    pub fn build_from_parts<'a, Query, Intervals, Data>(
        &'a mut self,
        query: Query,
        intervals: Intervals,
        data: Data,
    ) -> Result<()>
    where
        Query: IntoIterator<Item = &'a Interval<Idx>>,
        Intervals: IntoIterator<Item = &'a Interval<Idx>>,
        Data: IntoIterator<Item = &'a &'tree T>,
        Idx: 'a,
    {
        self.clear();
        sweep_line_into(
            query.into_iter(),
            intervals.into_iter(),
            data.into_iter(),
            &mut self.segments,
            &mut self.data,
        )
    }

    /// Returns all annotation segments generated by the query.
    /// Segments are guaranteed to be non-overlapping and sorted in ascending order by coordinate.
    #[inline]
    pub fn segments(&self) -> &[Interval<Idx>] {
        &self.segments
    }

    /// Returns the sets of unique data references associated with each segment.
    #[inline]
    pub fn data(&self) -> &[HashSet<&'tree T>] {
        &self.data
    }

    /// Returns an iterator over pairs of `(&Interval<Idx>, &HashSet<&'tree T>)`.
    /// Each pair represents an individual annotation segment and its corresponding set of
    /// overlapping data references. Segments are yielded in coordinate-sorted order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&Interval<Idx>, &HashSet<&'tree T>)> {
        self.segments.iter().zip(self.data.iter())
    }

    /// Returns `true` if no segments were generated.
    /// True is returned if the `build` method hasn't been called, if the query intervals were empty,
    /// or if the query intervals had zero length.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns the number of distinct segments generated.
    #[inline]
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Clears the segments and associated data, retaining allocated capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.segments.clear();
        self.data.clear();
    }

    pub fn recycle<'new_tree, NewIdx: PrimInt, NewT: Eq + Hash>(
        mut self,
    ) -> HitSegments<'new_tree, NewIdx, NewT> {
        self.clear();

        let (segments, data) = self.dissolve();
        let data = data.into_iter().map(|_| unreachable!()).collect();
        let segments = segments.into_iter().map(|_| unreachable!()).collect();

        HitSegments { segments, data }
    }
}

/// Stores the results of segmenting a batch of query intervals and their overlapping hits.
///
/// **Key Guarantees:**
/// <TODO>
///
/// **Performance:**
/// - Similar to `HitSegments`, calculating segments for each query involves a sweep-line
///   algorithm (roughly O((N+M)*log(N+M)) per query), where N is the number of query intervals
///   and M is the number of hits.
/// - **Crucially**, like `HitSegments`, this structure clones the `HashSet<&'tree T>` for
///   *each segment generated across all queries*. If dealing with a large number of queries
///   or queries generating many segments with large overlap sets, this cloning can become
///   a significant performance and memory bottleneck.
///
/// **Buffer Reuse:**
/// - `clear()`: Removes all segments and data, keeping the allocated memory for reuse
///   within the same lifetime (`'tree`).
/// - `recycle()`: Consumes the object, returning a new empty one with the same allocated
///   capacity but potentially associated with new types/lifetime.
#[derive(Clone, PartialEq, Eq, Debug, Dissolve)]
pub struct BatchHitSegments<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> {
    // Flattened vector of all non-overlapping, sorted segments across all queries.
    segments: Vec<Interval<Idx>>,
    // Flattened vector of unique data sets corresponding to each segment.
    data: Vec<HashSet<&'tree T>>,
    // Stores boundaries for each query's results in the flattened vectors.
    // Results for query `i` are in the range `index[i]..index[i+1]`.
    index: Vec<usize>,
}

impl<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> Default for BatchHitSegments<'tree, Idx, T> {
    #[inline]
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            data: Vec::new(),
            index: vec![0], // Start with the boundary for the first (not yet added) query
        }
    }
}

impl<'tree, Idx: PrimInt, T: Eq + Hash + ?Sized> BatchHitSegments<'tree, Idx, T> {
    /// Creates a new, empty `BatchHitSegments` collection with no pre-allocated capacity.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates new, empty `BatchHitSegments` with pre-allocated capacity.
    ///
    /// # Arguments
    /// * `queries` - An estimate for the number of queries expected to be added.
    /// * `segments` - An estimate for the *total* number of segments across all queries.
    #[inline]
    pub fn with_capacity(queries: usize, total_segments: usize) -> Self {
        let mut index = Vec::with_capacity(queries + 1);
        index.push(0);

        Self {
            segments: Vec::with_capacity(total_segments),
            data: Vec::with_capacity(total_segments),
            index,
        }
    }

    /// Calculates the segments for each query and its hits using a sweep-line approach.
    ///
    /// Overwrites any existing segments and data stored in this `BatchHitSegments` object.
    ///
    /// # Arguments
    /// * `queries` – Intervals defining the regions of interest for each query.
    /// * `hits` – A reference to the `Hits` object containing the overlaps between query regions
    ///            and hits (intervals and associated data references `&'tree T`) found by querying
    ///            an interval tree.
    pub fn build<'a, Queries>(
        &mut self,
        queries: Queries,
        hits: &'a BatchHits<'tree, Idx, T>,
    ) -> Result<()>
    where
        Queries: IntoIterator<Item: IntoIterator<Item = &'a Interval<Idx>>>,
        Idx: 'a,
    {
        let mut queries = queries.into_iter();
        self.clear();

        for (q, h) in queries.by_ref().zip(hits.iter()) {
            sweep_line_into(
                q.into_iter(),
                h.0.into_iter(),
                h.1.into_iter(),
                &mut self.segments,
                &mut self.data,
            )?;
            self.index.push(self.segments.len());
        }
        ensure!(
            self.len() == hits.len() && queries.next().is_none(),
            "Mismatch between number of queries and hits."
        );
        Ok(())
    }

    /// Returns the segments generated for the `i`-th query in the batch.
    pub fn segments(&self, i: usize) -> Option<&[Interval<Idx>]> {
        if i + 1 < self.len() {
            Some(&self.segments[self.index[i]..self.index[i + 1]])
        } else {
            None
        }
    }

    /// Returns an iterator over the segments for each query in the batch.
    pub fn segments_iter(&self) -> impl ExactSizeIterator<Item = &[Interval<Idx>]> {
        (0..self.index.len() - 1).map(|i| &self.segments[self.index[i]..self.index[i + 1]])
    }

    /// Returns the data sets associated with segments for the `i`-th query in the batch.
    pub fn data(&self, i: usize) -> Option<&[HashSet<&'tree T>]> {
        if i + 1 < self.len() {
            Some(&self.data[self.index[i]..self.index[i + 1]])
        } else {
            None
        }
    }

    /// Returns an iterator over the data associated with segments of each query in the batch.
    pub fn data_iter(&self) -> impl ExactSizeIterator<Item = &[HashSet<&'tree T>]> {
        (0..self.index.len() - 1).map(|i| &self.data[self.index[i]..self.index[i + 1]])
    }

    /// Returns an iterator over the results for each query.
    /// Yields tuples `(&[Interval<Idx>], &[HashSet<&'tree T>])`, one tuple per query added.
    /// Yields empty slices for queries that had no hits or were empty.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&[Interval<Idx>], &[HashSet<&'tree T>])> {
        (0..self.index.len() - 1).map(|i| {
            (
                &self.segments[self.index[i]..self.index[i + 1]],
                &self.data[self.index[i]..self.index[i + 1]],
            )
        })
    }

    /// Returns `true` if no query results have been added to the batch yet.
    #[inline]
    pub fn is_empty(&self) -> bool {
        // Check if only the initial '0' boundary exists
        self.index.len() == 1
    }

    /// Returns the number of queries in the batch.
    #[inline]
    pub fn len(&self) -> usize {
        self.index.len() - 1
    }

    /// Returns the total number of segments across all queries in the batch.
    #[inline]
    pub fn total_segments(&self) -> usize {
        self.segments.len()
    }

    /// Clears the collection, removing all segments, data sets, and query counts,
    /// but retaining the allocated capacity for reuse.
    /// Use this for buffer reuse *within* the same lifetime scope `'tree`.
    #[inline]
    pub fn clear(&mut self) {
        self.segments.clear();
        self.data.clear();
        self.index.clear();
        self.index.push(0); // Reset to initial state
    }

    #[inline]
    pub fn recycle<'new_tree, NewIdx: PrimInt, NewT: Eq + Hash>(
        mut self,
    ) -> BatchHitSegments<'new_tree, NewIdx, NewT> {
        self.clear();

        let (segments, data, index) = self.dissolve();
        let data = data.into_iter().map(|_| unreachable!()).collect();
        let segments = segments.into_iter().map(|_| unreachable!()).collect();

        BatchHitSegments {
            segments,
            data,
            index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    pub fn assert_segments<'a, Idx: PrimInt + 'a>(
        segments: &HitSegments<'a, Idx, str>,
        expected: impl IntoIterator<Item = &'a (Idx, Idx, Vec<&'a str>)>,
    ) {
        let (intervals, data): (Vec<Interval<Idx>>, Vec<HashSet<&'a str>>) = expected
            .into_iter()
            .map(|(start, end, data)| {
                (
                    Interval::new(*start, *end).unwrap(),
                    HashSet::from_iter(data.into_iter().cloned()),
                )
            })
            .unzip();

        assert_eq!(
            segments.iter().collect_vec(),
            intervals.iter().zip(data.iter()).collect_vec()
        );
        assert_eq!(segments.segments(), &intervals);
        assert_eq!(segments.data(), &data);
    }

    pub fn build_segments<'a, Idx: PrimInt + 'a>(
        query: impl IntoIterator<Item = &'a (Idx, Idx)>,
        hits: impl IntoIterator<Item = &'a (Idx, Idx, &'static str)>,
    ) -> HitSegments<'static, Idx, str> {
        let mut allhits = Hits::new();
        allhits.extend(
            hits.into_iter()
                .cloned()
                .map(|(start, end, data)| (Interval::new(start, end).unwrap(), data)),
        );

        let mut segments = HitSegments::new();
        let query: Vec<_> = query
            .into_iter()
            .cloned()
            .map(|(start, end)| Interval::new(start, end).unwrap())
            .collect();
        segments.build(&query, &allhits).unwrap();
        segments
    }

    #[test]
    fn test_segments_new() {
        let segments = HitSegments::<'static, u8, ()>::new();
        assert!(segments.is_empty());
        assert_eq!(segments.len(), 0);
        assert_eq!(segments.iter().count(), 0);
    }

    #[test]
    fn test_segments_single_interval_no_hits() {
        let query = [(0, 10)];
        let expected = [(0, 10, vec![])];

        for hits in [
            [].as_slice(),
            [(20, 30, "A")].as_slice(),
            [(10, 30, "A"), (20, 50, "B")].as_slice(),
        ] {
            let segments = build_segments(&query, hits);
            assert_segments(&segments, &expected);
        }
    }

    #[test]
    fn test_segments_multi_interval_no_hits() {
        let query = [(10, 20), (25, 35)];
        let expected = [(10, 20, vec![]), (25, 35, vec![])];

        for hits in [
            [].as_slice(),
            [(0, 10, "1")].as_slice(),
            [(20, 25, "2")].as_slice(),
            [(35, 40, "3")].as_slice(),
            [(0, 10, "4-1"), (20, 25, "4-2"), (35, 40, "4-3")].as_slice(),
        ] {
            let segments = build_segments(&query, hits);
            assert_segments(&segments, &expected);
        }
    }

    #[test]
    fn test_segments_single_interval_single_hit() {
        let query = [(10, 20)];

        for (hits, expected) in [
            (
                vec![(5, 15, "A")],
                vec![(10, 15, vec!["A"]), (15, 20, vec![])],
            ),
            (
                vec![(10, 12, "A")],
                vec![(10, 12, vec!["A"]), (12, 20, vec![])],
            ),
            (
                vec![(12, 16, "A")],
                vec![(10, 12, vec![]), (12, 16, vec!["A"]), (16, 20, vec![])],
            ),
            (
                vec![(18, 20, "A")],
                vec![(10, 18, vec![]), (18, 20, vec!["A"])],
            ),
            (vec![(0, 100, "A")], vec![(10, 20, vec!["A"])]),
        ] {
            let segments = build_segments(&query, &hits);
            assert_segments(&segments, &expected);
        }
    }

    #[test]
    fn test_segments_multiple_interval_multiple_hits_1() {
        let query = [(10, 20), (50, 80)];

        for (hits, expected) in [
            (
                vec![(5, 15, "A"), (60, 70, "B")],
                vec![
                    (10, 15, vec!["A"]),
                    (15, 20, vec![]),
                    (50, 60, vec![]),
                    (60, 70, vec!["B"]),
                    (70, 80, vec![]),
                ],
            ),
            (
                vec![(40, 100, "A")],
                vec![(10, 20, vec![]), (50, 80, vec!["A"])],
            ),
            (
                vec![(8, 22, "X"), (45, 55, "Y"), (60, 80, "Z")],
                vec![
                    (10, 20, vec!["X"]),
                    (50, 55, vec!["Y"]),
                    (55, 60, vec![]),
                    (60, 80, vec!["Z"]),
                ],
            ),
            (
                vec![
                    (12, 18, "A1"),
                    (15, 20, "A2"),
                    (65, 70, "B1"),
                    (75, 90, "B2"),
                ],
                vec![
                    (10, 12, vec![]),
                    (12, 15, vec!["A1"]),
                    (15, 18, vec!["A1", "A2"]),
                    (18, 20, vec!["A2"]),
                    (50, 65, vec![]),
                    (65, 70, vec!["B1"]),
                    (70, 75, vec![]),
                    (75, 80, vec!["B2"]),
                ],
            ),
            (
                vec![(5, 15, "A"), (20, 30, "B"), (35, 45, "C"), (50, 60, "D")],
                vec![
                    (10, 15, vec!["A"]),
                    (15, 20, vec![]),
                    (50, 60, vec!["D"]),
                    (60, 80, vec![]),
                ],
            ),
        ] {
            let segments = build_segments(&query, &hits);
            assert_segments(&segments, &expected);
        }
    }

    #[test]
    fn test_segments_single_interval_multiple_hits_2() {
        let query = [(0, 10), (20, 30)];
        let hits = vec![
            (0, 100, "A"),
            (0, 2, "A"),
            (2, 10, "A"),
            (20, 25, "A"),
            (25, 30, "A"),
        ];
        let expected = vec![(0, 10, vec!["A"]), (20, 30, vec!["A"])];
        let segments = build_segments(&query, &hits);
        assert_segments(&segments, &expected);
    }

    #[test]
    fn test_segments_multiple_interval_multiple_hits_3() {
        let query = [(2, 5), (6, 8), (100, 120)];
        let mut hits = vec![
            (1, 9, "1-9"),
            (2, 7, "2-7"),
            (3, 5, "3-5"),
            (4, 6, "4-6"),
            (4, 12, "4-12"),
            (7, 9, "7-9"),
        ];
        let expected = vec![
            (2, 3, vec!["1-9", "2-7"]),
            (3, 4, vec!["1-9", "2-7", "3-5"]),
            (4, 5, vec!["1-9", "2-7", "3-5", "4-6", "4-12"]),
            (6, 7, vec!["1-9", "2-7", "4-12"]),
            (7, 8, vec!["1-9", "7-9", "4-12"]),
            (100, 120, vec![]),
        ];
        let segments = build_segments(&query, &hits);
        assert_segments(&segments, &expected);

        hits.extend([
            (2, 4, "A"),
            (2, 7, "A"),
            (5, 10, "A"),
            (1, 3, "C"),
            (5, 7, "C"),
            (7, 8, "C"),
        ]);
        let expected = vec![
            (2, 3, vec!["1-9", "2-7", "A", "C"]),
            (3, 4, vec!["1-9", "2-7", "3-5", "A"]),
            (4, 5, vec!["1-9", "2-7", "3-5", "4-6", "4-12", "A"]),
            (6, 7, vec!["1-9", "2-7", "4-12", "A", "C"]),
            (7, 8, vec!["1-9", "7-9", "4-12", "A", "C"]),
            (100, 120, vec![]),
        ];
        let segments = build_segments(&query, &hits);
        assert_segments(&segments, &expected);
    }

    #[test]
    fn test_batch_segments_new() {
        let batch = BatchHitSegments::<'static, u8, str>::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
        assert_eq!(batch.total_segments(), 0);
        assert_eq!(batch.segments_iter().count(), 0);
        assert_eq!(batch.data_iter().count(), 0);
        assert_eq!(batch.iter().count(), 0);
        assert_eq!(batch.segments(0), None);
        assert_eq!(batch.data(0), None);
        assert_eq!(batch.index, vec![0]);
        assert_eq!(batch, BatchHitSegments::default());

        let batch_cap = BatchHitSegments::<'static, u8, str>::with_capacity(5, 50);
        assert!(batch_cap.is_empty());
        assert_eq!(batch_cap.len(), 0);
        assert_eq!(batch_cap.index, vec![0]);
        assert!(batch_cap.segments.capacity() >= 50);
        assert!(batch_cap.data.capacity() >= 50);
        assert!(batch_cap.index.capacity() >= 6); // 5 queries + 1
    }

    #[test]
    fn test_batch_segments_build() -> Result<()> {
        let mut batch = BatchHits::new();
        {
            let mut hits = batch.add_hits();
            hits.add(Interval::new(0, 10)?, "A");
            hits.add(Interval::new(5, 15)?, "B");
        }
        {
            let mut hits = batch.add_hits();
            hits.add(Interval::new(20, 30)?, "C");
            hits.add(Interval::new(25, 35)?, "D");
        }

        let mut segments = BatchHitSegments::new();
        segments.build(
            [
                &vec![Interval::new(0, 10)?],
                &vec![Interval::new(20, 25)?, Interval::new(30, 40)?],
            ],
            &batch,
        )?;

        assert_eq!(
            segments.iter().collect_vec(),
            vec![
                (
                    [Interval::new(0, 5)?, Interval::new(5, 10)?].as_slice(),
                    [HashSet::from_iter(["A"]), HashSet::from_iter(["A", "B"])].as_slice()
                ),
                (
                    [
                        Interval::new(20, 25)?,
                        Interval::new(30, 35)?,
                        Interval::new(35, 40)?
                    ]
                    .as_slice(),
                    [
                        HashSet::from_iter(["C"]),
                        HashSet::from_iter(["D"]),
                        HashSet::new()
                    ]
                    .as_slice()
                )
            ]
        );

        segments.clear();
        assert!(segments.is_empty());
        assert_eq!(segments.len(), 0);
        assert_eq!(segments.total_segments(), 0);
        assert_eq!(segments.index, vec![0]);
        assert_eq!(segments.segments(0), None);
        assert_eq!(segments.data(0), None);
        assert_eq!(segments.segments_iter().count(), 0);
        assert_eq!(segments.data_iter().count(), 0);
        assert_eq!(segments.iter().count(), 0);
        Ok(())
    }
}

// fn _build_hashed(&mut self, query: &[Interval<Idx>], hits: &Hits<'tree, Idx, T>) {
// // boundaries are of length N + 1
// // annotation is of length N
// // N is recorded in the hitlen for each query
// self.clear();
// let mut total = 0;
// for (query, (hits, annotations)) in data {
//     self.cache.clear();
//     self.cache.insert(query.start());
//     self.cache.insert(query.end());
//     for it in hits.iter() {
//         if it.start() > query.start() {
//             self.cache.insert(it.start());
//         }
//         if it.end() < query.end() {
//             self.cache.insert(it.end());
//         }
//     }
//
//     self.boundaries.extend(self.cache.iter());
//     self.hitlen.push(self.cache.len() - 1);
//
//     // Allocate enough space for all the annotations if needed
//     if self.annotation.len() < total + self.cache.len() - 1 {
//         self.annotation
//             .resize_with(total + self.cache.len() - 1, || HashSet::new());
//     }
//
//     // Populate stepped annotation for the current query
//     let boundaries = &self.boundaries[total + self.hitlen.len() - 1..];
//     let steps = &mut self.annotation[total..total + self.cache.len() - 1];
//     debug_assert!(boundaries.len() == steps.len() + 1);
//     for (it, anno) in hits.iter().zip(annotations) {
//         let st = if it.start() <= query.start() {
//             0
//         } else {
//             boundaries.binary_search(&it.start()).unwrap()
//         };
//         let en = if it.end() >= query.end() {
//             steps.len()
//         } else {
//             boundaries.binary_search(&it.end()).unwrap()
//         };
//
//         for step in steps[st..en].iter_mut() {
//             step.insert(anno.clone());
//         }
//     }
//
//     total += self.cache.len() - 1;
// }
//
// debug_assert!(total == self.boundaries.len() - self.hitlen.len());
// debug_assert!(total == self.hitlen.iter().sum());
// }
