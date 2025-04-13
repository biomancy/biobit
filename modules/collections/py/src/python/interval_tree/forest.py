from collections.abc import MutableMapping, Mapping
from typing import Iterator

from . import Bits, BitsBuilder


class Forest[K, V](MutableMapping[K, Bits[V]]):
    """
    A collection of pre-built interval trees (Bits), partitioned by arbitrary keys.

    Acts like a dictionary where keys map to individual, already constructed
    `biobit.collections.interval_tree.Bits` instances. This class does *not*
    build the trees; they must be provided pre-built.

    Implements the `MutableMapping` protocol for standard dictionary operations.
    """
    _trees: dict[K, Bits[V]]

    def __init__(self, trees: Mapping[K, Bits[V] | BitsBuilder[V]] | None = None):
        """
        Initialize the Forest.

        Args:
            trees: An optional dictionary to pre-populate the Forest.
        """
        self._trees: dict[K, Bits[V]] = {}
        if trees is not None:
            for key, value in trees.items():
                if isinstance(value, BitsBuilder):
                    value = value.build()
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
