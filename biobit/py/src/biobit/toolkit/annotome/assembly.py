from collections.abc import Mapping
from types import MappingProxyType
from typing import Iterable, Self, Iterator

from biobit.toolkit.annotome.annotation.annotation import Annotation
from .reference import RefRegistry


class AnnotationMap(Mapping[str, Annotation]):
    """
    A dict-like, read-only view of annotations with strict typing capabilities.
    """
    __slots__ = ("_store",)
    _store: Mapping[str, Annotation]

    def __init__(self, store: Mapping[str, Annotation]):
        self._store = store

    def fetch[T: Annotation](self, key: str, target: type[T]) -> T:
        """
        Strict retrieval.

        Returns the annotation if it exists AND matches the target type.

        Raises:
            KeyError: If key is missing.
            TypeError: If key exists but type does not match target.
        """
        val = self._store.get(key)
        if val is None:
            raise KeyError(f"Annotation '{key}' not found.")
        if isinstance(val, target):
            return val
        raise TypeError(
            f"Annotation '{key}' is {type(val).__name__}, expected {target.__name__}."
        )

    def __getitem__(self, key: str) -> Annotation:
        return self._store[key]

    def __iter__(self) -> Iterator[str]:
        return iter(self._store)

    def __len__(self) -> int:
        return len(self._store)


class Assembly:
    """A formal container for a genomic assembly and its annotation."""

    def __init__(self, name: str, organisms: Iterable[str], refrg: RefRegistry, annotations: Mapping[str, Annotation]):
        self._name: str = name
        self._organisms: frozenset[str] = frozenset(organisms)
        self._refrg: RefRegistry = refrg
        self._annotations: dict[str, Annotation] = dict(annotations)

    @property
    def name(self) -> str:
        """A unique name for the assembly (e.g., 'GRCh38')."""
        return self._name

    @property
    def organisms(self) -> frozenset[str]:
        """Organisms covered by the assembly (e.g., 'Homo sapiens', 'HSV-1')."""
        return self._organisms

    @property
    def references(self) -> RefRegistry:
        return self._refrg

    @property
    def annotations(self) -> AnnotationMap:
        return AnnotationMap(MappingProxyType(self._annotations))

    @classmethod
    def merge(cls, name: str, items: Iterable[Self]) -> "Assembly":
        """
        Merge given assemblies into a single unified Assembly.

        Merges:
        1. Organisms (Union of sets)
        2. References (Concatenation via RefRegistry.merge)
        3. Annotations (Group by key -> Annotation.merge)
        """
        items = tuple(items)
        if not items:
            raise ValueError("Assembly.merge called with an empty collection of assemblies")

        # Merge organisms description.
        # Here we allow repeated organism as one might be interested in studying a metagenome of an E. coli culture.
        # Thus, more than one assembly might cover the same organism
        organisms: set[str] = set()
        for it in items:
            organisms.update(it._organisms)

        # Merge reference registries
        refrg = RefRegistry.merge(x._refrg for x in items)

        # Merge Annotations by key
        allkeys: set[str] = set()
        for it in items:
            allkeys.update(it._annotations.keys())

        grouped: dict[str, list[tuple[RefRegistry, Annotation | None]]] = {key: [] for key in allkeys}
        for it in items:
            for key in allkeys:
                if key in it._annotations:
                    grouped[key].append((it._refrg, it._annotations[key]))
                else:
                    grouped[key].append((it._refrg, None))

        annotations = {}
        for key, values in grouped.items():
            # Ensure that all annotations are of the same type (or None)
            types = {type(v) for _, v in values if v is not None}
            if len(types) > 1:
                raise TypeError(
                    f"Cannot merge heterogeneous annotations for key '{key}': "
                    f"{set(t.__name__ for t in types)}"
                )

            # Annotations can't be all None, as at least one assembly must have the annotation
            assert len(types) == 1
            atype = types.pop()
            annotations[key] = atype.merge(refrg, values)
        return Assembly(name, organisms, refrg, annotations)
