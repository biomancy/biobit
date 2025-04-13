from typing import Iterator, Any

from biobit.core.loc import Interval


class HitSegments[T]:
    """
    Represents the segmentation of query intervals based on overlapping hits.

    Stores non-overlapping, sorted segments covering the query regions,
    each associated with the unique set of data objects whose intervals
    overlap that specific segment.
    """

    def __init__(self) -> None:
        """Create a new, empty HitSegments container."""
        ...

    def segments(self) -> list[Interval]:
        """
        Returns a list of all segmentation intervals.

        Segments are non-overlapping and sorted.

        Returns:
            list[Interval]: The list of segments. Note: Creates a copy.
        """
        ...

    def data(self) -> list[frozenset[T]]:
        """
        Returns a list of frozensets, where each set contains the unique data
        objects overlapping the corresponding segment.

        Returns:
            list[frozenset[T]]: List of frozensets. Note: Creates a copy of the list structure.
                                  The elements within the frozensets are the original Python objects.
        """
        ...

    def clear(self) -> None:
        """Clear the stored segments and associated data sets."""
        ...

    def __len__(self) -> int:
        """
        Returns the number of segments.

        Returns:
            int: The number of segments.
        """
        ...

    def __iter__(self) -> Iterator[tuple[Interval, frozenset[T]]]:
        """
        Iterate over the segments as (segment_interval, data_frozenset) pairs.

        Yields:
            tuple[Interval, frozenset[T]]: A tuple containing the segment interval
                                               and the frozenset of data objects overlapping it.
        """
        ...

    def __eq__(self, other: object) -> bool: ...

    def __getstate__(self) -> Any: ...

    def __setstate__(self, state: Any) -> None: ...


class BatchHitSegments[T]:
    """
    Stores the results of segmenting multiple query intervals based on overlapping hits.

    Efficiently stores segmentation results (segments and their associated data sets)
    for multiple queries processed in a batch.
    """

    def __init__(self) -> None:
        """Create a new, empty BatchHitSegments container."""
        ...

    def segments(self, i: int) -> list[Interval]:
        """
        Get the segments for a specific query index.

        Args:
            i: The 0-based index of the query.

        Returns:
            list[Interval]: A list of segments for the specified query. Note: Creates a copy.

        Raises:
            IndexError: If the query index `i` is out of bounds.
        """
        ...

    def data(self, i: int) -> list[frozenset[T]]:
        """
        Get the data frozensets for a specific query index.

        Args:
            i: The 0-based index of the query.

        Returns:
            list[frozenset[T]]: A list of frozensets for the specified query. Note: Creates a copy.

        Raises:
            IndexError: If the query index `i` is out of bounds.
        """
        ...

    def clear(self) -> None:
        """Clear stored segments for all queries."""
        ...

    def __len__(self) -> int:
        """
        Returns the number of queries stored in the batch.

        Returns:
            int: The number of queries processed.
        """
        ...

    def __iter__(self) -> Iterator[tuple[list[Interval], list[frozenset[T]]]]:
        """
        Iterate over the batch results, yielding segmentation results for each query.

        Yields:
            tuple[list[Interval], list[frozenset[T]]]:
                A tuple containing the list of segments and the list of data frozensets
                for one query.
        """
        ...

    def __eq__(self, other: object) -> bool: ...

    def __getstate__(self) -> Any: ...

    def __setstate__(self, state: Any) -> None: ...
