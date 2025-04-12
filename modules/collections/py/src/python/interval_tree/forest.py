from collections.abc import MutableMapping, Mapping
from typing import Any, Iterator

from biobit.core.loc import IntoInterval
from . import Bits, Hits, BatchHits


class Forest[K, V](MutableMapping[K, Bits[V]]):
    """
    A collection of pre-built interval trees (Bits), partitioned by arbitrary keys.

    Acts like a dictionary where keys map to individual, already constructed
    `biobit.collections.interval_tree.Bits` instances. This class does *not*
    build the trees; they must be provided pre-built.

    Implements the `MutableMapping` protocol for standard dictionary operations.

    Example:
        # Assume tree1 and tree2 are pre-built Bits[str] instances
        # tree1 = BitsBuilder(...).build()
        # tree2 = BitsBuilder(...).build()

        forest = Forest[str, str]({"chr1": tree1, "chr2": tree2})

        # Or add later:
        # forest["chr1"] = tree1

        hits_chr1 = forest.overlap("chr1", (18, 22))
        hits_chr2 = forest.batch_overlap("chr2", [(0, 10), (50, 60)])
    """
    _trees: dict[K, Bits[V]]

    def __init__(self, trees: Mapping[K, Bits[V]] | None = None):
        """
        Initialize the Forest.

        Args:
            trees: An optional dictionary to pre-populate the Forest.
                   Values MUST be already built Bits instances.
        """
        self._trees: dict[K, Bits[V]] = {}
        if trees is not None:
            for key, value in trees.items():
                self[key] = value

    def __setitem__(self, key: K, value: Bits[V]):
        """
        Set or replace the Bits tree associated with a key.

        Args:
            key: The partition key.
            value: The pre-built Bits[V] instance.
        """
        self._trees[key] = value

    def __getitem__(self, key: K) -> Bits[V]:
        """
        Get the Bits tree for a key. Raises KeyError if not found.
        """
        tree = self._trees.get(key)
        if tree is None:
            raise KeyError(f"Key '{key}' not found in Forest")
        return tree

    def __delitem__(self, key: K):
        """Delete the Bits tree associated with a key."""
        del self._trees[key]

    def __iter__(self) -> Iterator[K]:
        """Iterate over the keys (partitions) in the Forest."""
        return iter(self._trees)

    def __len__(self) -> int:
        """Return the number of partitions (keys) in the Forest."""
        return len(self._trees)

    def intersect(self, key: K, interval: IntoInterval, *, into: Hits[Any] | None = None) -> Hits[V]:
        """
        Find entries overlapping the query interval within a specific tree.

        Args:
            key: The key identifying the tree to query.
            interval: The query interval (e.g., (100, 200) or Interval(100, 200)).
            into: An optional existing Hits buffer to store results in,
                  overwriting its contents. If None, a new Hits object is created.

        Returns:
            A Hits object containing the overlapping intervals and their data.

        Raises:
            KeyError: If the key is not found in the Forest.
        """
        tree = self[key]
        hits: Hits[V] = tree.intersect_interval(interval, into=into)
        return hits

    def batch_intersect(
            self, key: K, intervals: list[IntoInterval], *, into: BatchHits[Any] | None = None
    ) -> BatchHits[V]:
        """
        Find entries overlapping each query interval in a batch within a specific tree.

        Args:
            key: The key identifying the tree to query.
            intervals: A list of query intervals.
            into: An optional existing BatchHits buffer to store results in,
                  overwriting its contents. If None, a new BatchHits object is created.

        Returns:
            A BatchHits object containing the results for each query interval.

        Raises:
            KeyError: If the key is not found in the Forest.
        """
        tree = self[key]
        bhits = tree.batch_intersect_intervals(intervals, into=into)
        return bhits
