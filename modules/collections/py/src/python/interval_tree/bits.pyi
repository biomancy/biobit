from typing import Any, Iterable

from biobit.core.loc import Interval, IntoInterval
from . import Hits, BatchHits


class BitsBuilder[T]:
    """A builder for constructing Bits interval trees."""

    def __init__(self) -> None:
        """Create a new builder instance."""
        ...

    def add(self, interval: IntoInterval, data: T) -> "BitsBuilder":
        """
        Add an interval and its corresponding Python object data.

        Args:
            interval: The interval (Interval, tuple, or list).
            data: The Python object associated with the interval.

        Returns:
            Itself, for chaining.
        """
        ...

    def extend(self, data: Iterable[tuple[IntoInterval, T]]) -> "BitsBuilder":
        """
        Extend the builder from an iterable of (interval, data) pairs.

        Args:
            data: An iterable yielding (interval, data) tuples.
                  Intervals can be Interval, tuple, or list.

        Returns:
            Itself, for chaining.
        """
        ...

    def build(self) -> "Bits":
        """
        Build the final Bits interval tree.

        Returns:
            The constructed interval tree.
        """
        ...


class Bits[T]:
    """
    An interval tree implementation using the BITS algorithm.

    Stores intervals associated with Python data objects using internal indices.
    Efficient for queries where intervals are relatively short compared to the
    coordinate space. The tree structure (intervals and object indices) is
    immutable after construction.
    """

    @staticmethod
    def builder() -> BitsBuilder:
        """
        Create a new builder for constructing a Bits interval tree.
        """
        ...

    def data(self) -> list[T]:
        """
        Get the list of data objects stored in the tree.
        """
        ...

    def intervals(self) -> list[Interval]:
        """
        Get the list of intervals stored in the tree.
        """
        ...

    def records(self) -> list[tuple[Interval, T]]:
        """
        Get the list of (interval, data) records stored in the tree.
        """
        ...

    def intersect_interval(self, interval: IntoInterval, into: Hits[Any] | None = None) -> Hits[T]:
        """
        Find entries overlapping the query interval.

        Args:
            interval: The query interval (Interval, tuple, or list).
            into: An optional existing Hits buffer to store results in,
                  overwriting its contents. If None, a new Hits object is created.

        Returns:
            The Hits object containing the results.
        """
        ...

    def batch_intersect_intervals(
            self, intervals: list[IntoInterval], into: BatchHits[Any] | None = None
    ) -> BatchHits[T]:
        """
        Find entries overlapping each query interval in a batch.
        That is, each interval serves as a separate query.

        Args:
            intervals: A list of query intervals (Interval, tuple, or list).
            into: An optional existing BatchHits buffer to store results in,
                  overwriting its contents.
                  If None, a new BatchHits object is created.

        Returns:
            The BatchHits object containing the results.
        """
        ...

    def __eq__(self, other: object) -> bool: ...
