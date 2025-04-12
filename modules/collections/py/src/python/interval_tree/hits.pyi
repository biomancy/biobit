from collections.abc import Sequence
from typing import Any, Iterator, Iterable

from biobit.core.loc import Interval, IntoInterval
from .segments import HitSegments, BatchHitSegments


class Hits[T]:
    """
    Stores the results of a single interval tree query.

    Contains intervals and associated data objects found in the tree
    that overlap the query region.
    """

    def __init__(self) -> None:
        """Create a new, empty Hits container."""
        ...

    def append(self, interval: IntoInterval, data: T) -> None:
        """
        Append a single hit (interval and data).
        """
        ...

    def extend(self, items: Iterable[tuple[IntoInterval, T]]) -> None:
        """
        Extend the list of hits from an iterable of (interval, data) pairs.
        """
        ...

    def intervals(self) -> list[Interval]:
        """
        Returns a list of all intervals found.

        Note: This method typically returns a *new list* (copy) each time it's called.

        Returns:
            list[Interval]: A list of interval objects.
        """
        ...

    def data(self) -> list[T]:
        """
        Returns a list of data associated with the found intervals.

        Note: This method typically returns a *new list* (copy) containing references
              to the original Python objects found during the query.

        Returns:
            list[T]: A list of Python objects associated with each hit.
        """
        ...

    def segment(
            self,
            query: Sequence[IntoInterval],
            into: HitSegments[T] | None = None,
    ) -> HitSegments[T]:
        """
        Calculates segmentation based on these hits and the provided query intervals.

        Segments the regions defined by `query` based on which hits from this
        container overlap each part of the query region.

        Args:
            query: A sequence of intervals defining the regions of interest
                   for segmentation (e.g., list[Interval], list[tuple[int, int]]).
            into: An optional existing HitSegments object to reuse for storing
                  results, potentially improving performance by avoiding reallocation.
                  If None, a new HitSegments object is created.

        Returns:
            HitSegments[T]: The segmentation results. This will be the `into`
                             object if provided, otherwise a new object.

        Raises:
            ValueError: If query intervals are invalid or if hits/data counts mismatch internally
                        (propagated from Rust). # Added Raises
        """
        ...

    def clear(self) -> None:
        """Clear the stored hits, removing all intervals and data."""
        ...

    def __len__(self) -> int:
        """
        Returns the number of hits found.

        Returns:
            int: The total number of hits.
        """
        ...

    def __iter__(self) -> Iterator[tuple[Interval, T]]:
        """
        Iterate over the hits as (interval, data) pairs.

        Yields:
            tuple[Interval, T]: A tuple containing the interval and its associated data object.
        """
        ...


class BatchHits[T]:
    """
    Stores the results of multiple interval tree queries (a batch).

    Results for individual queries are accessible via methods like `intervals` and `data`.
    """

    def __init__(self) -> None:
        """Create a new, empty BatchHits container."""
        ...

    def intervals(self, i: int) -> list[Interval]:
        """
        Get the intervals for a specific query index from the batch.

        Note: This method typically returns a *new list* (copy).

        Args:
            i: The 0-based index of the query in the batch.

        Returns:
            list[Interval]: A list of intervals for the specified query.

        Raises:
            IndexError: If the query index `i` is out of bounds.
        """
        ...

    def data(self, i: int) -> list[T]:
        """
        Get the data associated with hits for a specific query index from the batch.

        Note: This method typically returns a *new list* (copy) containing references
              to the original Python objects.

        Args:
            i: The 0-based index of the query in the batch.

        Returns:
            list[T]: A list of Python objects for the specified query.

        Raises:
            IndexError: If the query index `i` is out of bounds.
        """
        ...

    def append(self, intervals: Iterable[IntoInterval], data: Iterable[T]) -> None:
        """
        Append results for a single new query to the end of the batch.
        """
        ...

    def extend(self, queries: Iterable[tuple[Iterable[IntoInterval], Iterable[T]]]) -> None:
        """
        Extend the batch with results from multiple queries provided via an iterable.
        """
        ...

    def segment(
            self,
            query: Sequence[Sequence[IntoInterval]],
            into: BatchHitSegments[T] | None = None,
    ) -> BatchHitSegments[T]:
        """
        Calculates segmentation for each query in the batch.

        Segments the regions defined by each sub-sequence in `query` based
        on which hits from the corresponding batch entry overlap each part
        of the query region.

        Args:
            query: A sequence of sequences of intervals. Each inner sequence
                   defines the regions of interest for segmentation for the
                   corresponding query in the batch.
            into: An optional existing BatchHitSegments object to reuse for storing
                  results, potentially improving performance by avoiding reallocation.
                  If None, a new BatchHitSegments object is created.

        Returns:
            BatchHitSegments[T]: The batch segmentation results. This will be the
                                  `into` object if provided, otherwise a new object.
        """
        ...

    def clear(self) -> None:
        """Clear stored hits for all queries."""
        ...

    def __len__(self) -> int:
        """
        Returns the number of queries stored in the batch.

        Returns:
            int: The number of queries processed.
        """
        ...

    def __iter__(self) -> Iterator[tuple[list[Interval], list[T]]]:
        """
        Iterate over the batch results, yielding results for each query.

        Yields:
            tuple[list[Interval], list[T]]:
                A tuple containing the list of intervals and the list of data for each query.
        """
        ...

    def __eq__(self, other: object) -> bool: ...

    def __getstate__(self) -> Any: ...

    def __setstate__(self, state: Any) -> None: ...
