from .interval_tree import overlap

type IntoBundle[K, V] = dict[K, list[V]] | dict[K, tuple[V, ...]]

__all__ = ["IntoBundle", "overlap"]
