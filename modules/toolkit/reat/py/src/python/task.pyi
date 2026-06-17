from collections.abc import Sequence

from biobit.core.loc import Interval, IntoInterval

class Task:
    """
    A set of intervals processed as one batch by REAT workers.

    Coordinates are converted to unsigned 64-bit integers.
    """

    seqid: str
    """Sequence/contig identifier."""

    envelope: Interval
    """Smallest half-open interval covering all task intervals."""

    intervals: list[Interval]
    """Sorted non-overlapping half-open intervals of interest within the envelope."""

    def __init__(self, seqid: str, intervals: Sequence[IntoInterval]) -> None:
        """
        Create a task from already sorted, non-overlapping intervals.

        Intervals must be non-empty. Adjacent intervals are allowed, but overlapping intervals are
        rejected here; use `from_intervals()` to sort and merge input intervals first.
        """
        ...

    @staticmethod
    def from_intervals(
        intervals: Sequence[tuple[str, IntoInterval]], max_task_size: int
    ) -> list[Task]:
        """
        Build tasks from arbitrary `(seqid, interval)` pairs.

        Input is grouped by `seqid`, sorted by `seqid`, and intervals per sequence are merged.
        Touching intervals, e.g. `[0, 10)` and `[10, 20)`, are merged as well as overlapping
        intervals. Large merged intervals are split into chunks no larger than `max_task_size`.
        """
        ...

    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore
