import heapq
from collections import Counter
from collections.abc import Mapping, Iterator, Iterable
from typing import Self


class PropertyMap[Key, Value](Mapping[Key, Value]):
    """A proxy view of a specific attribute (column) across the registry."""
    __slots__ = ("_key2idx", "_values")

    def __init__(self, key2idx: dict[Key, int], values: tuple[Value, ...]):
        super().__init__()
        self._key2idx = key2idx
        self._values = values

    def __getitem__(self, key: Key) -> Value:
        idx = self._key2idx.get(key)
        if idx is None:
            raise KeyError(f"Key '{key}' not found.")
        return self._values[idx]

    def __iter__(self) -> Iterator[Key]:
        return iter(self._key2idx)

    def __len__(self) -> int:
        return len(self._key2idx)


class Reference:
    """Data Transfer Object defining a reference sequence."""
    __slots__ = ("name", "length", "label", "description", "aliases")

    def __init__(
            self,
            name: str,
            length: int,
            label: str | None = None,
            description: str | None = None,
            aliases: Iterable[str] | None = None
    ):
        if length <= 0:
            raise ValueError(f"Reference '{name}' must have a positive length.")
        self.name = name
        self.length = length
        self.label = label if label is not None else name
        self.description = description
        self.aliases = frozenset(aliases) if aliases is not None else frozenset()

        if name in self.aliases:
            raise ValueError(f"Reference name '{name}' cannot be its own alias.")

    def __hash__(self):
        return hash((self.name, self.length, self.label, self.description, self.aliases))

    def __eq__(self, other):
        if isinstance(other, (Reference, ReferenceProxy)):
            return (self.name, self.length, self.label, self.description, self.aliases) == \
                (other.name, other.length, other.label, other.description, other.aliases)
        return NotImplemented


class ReferenceProxy:
    """A lightweight, read-only handle to a reference stored in a RefRegistry."""
    __slots__ = ("_idx", "_registry")

    def __init__(self, idx: int, registry: "RefRegistry"):
        self._idx = idx
        self._registry = registry

    @property
    def name(self) -> str:
        return self._registry._names[self._idx]

    @property
    def length(self) -> int:
        return self._registry._lengths[self._idx]

    @property
    def label(self) -> str:
        return self._registry._labels[self._idx]

    @property
    def description(self) -> str | None:
        return self._registry._descriptions[self._idx]

    @property
    def aliases(self) -> frozenset[str]:
        return self._registry._aliases[self._idx]

    def __hash__(self):
        return hash((self.name, self.length, self.label, self.description, self.aliases))

    def __eq__(self, other):
        if isinstance(other, ReferenceProxy):
            # Fast path: same index in same registry
            if self._registry is other._registry:
                return self._idx == other._idx

        if isinstance(other, (Reference, ReferenceProxy)):
            return (self.name, self.length, self.label, self.description, self.aliases) == \
                (other.name, other.length, other.label, other.description, other.aliases)
        return False


class RefRegistry:
    """
    An immutable, high-performance registry of reference sequences.

    Acts as the canonical coordinate system for an Assembly.
    """
    __slots__ = ("_index", "_mapping", "_names", "_lengths", "_labels", "_descriptions", "_aliases")
    _index: dict[str, int]
    _mapping: dict[str, int]
    _names: tuple[str, ...]
    _lengths: tuple[int, ...]
    _labels: tuple[str, ...]
    _descriptions: tuple[str | None, ...]
    _aliases: tuple[frozenset[str], ...]

    def __init__(self, references: Iterable[Reference]):
        """Standard constructor from iterable of Reference objects."""
        # Sort by name to ensure deterministic order
        refs = sorted(references, key=lambda r: r.name)
        self._validate_integrity(refs)

        # Build SoA (Structure of Arrays)
        names, lengths, labels, descs, aliases = [], [], [], [], []

        # _index maps canonical name -> int
        # _mapping maps canonical AND aliases -> int
        index, mapping = {}, {}

        for idx, ref in enumerate(refs):
            index[ref.name] = idx
            mapping[ref.name] = idx
            for alias in ref.aliases:
                mapping[alias] = idx

            names.append(ref.name)
            lengths.append(ref.length)
            labels.append(ref.label)
            descs.append(ref.description)
            aliases.append(ref.aliases)

        RefRegistry._from_buffers(
            self, index, mapping, tuple(names), tuple(lengths), tuple(labels), tuple(descs), tuple(aliases)
        )

    @classmethod
    def _from_buffers(
            cls,
            self,
            index: dict[str, int],
            mapping: dict[str, int],
            names: tuple[str, ...],
            lengths: tuple[int, ...],
            labels: tuple[str, ...],
            descriptions: tuple[str | None, ...],
            aliases: tuple[frozenset[str], ...]
    ) -> Self:
        """
        Low-level constructor for direct SoA initialization.
        Useful for zero-copy construction from Rust or Arrow buffers.
        """
        if not (len(names) == len(lengths) == len(labels) == len(descriptions) == len(aliases)):
            raise ValueError("All attribute buffers must have the same length.")

        self._index = index
        self._mapping = mapping
        self._names = names
        self._lengths = lengths
        self._labels = labels
        self._descriptions = descriptions
        self._aliases = aliases
        return self

    @staticmethod
    def _validate_integrity(refs: Iterable[Reference]):
        names = Counter(r.name for r in refs)
        if repeated := [n for n, c in names.items() if c > 1]:
            raise ValueError(f"Duplicate canonical names: {repeated[:5]}")

        # Check aliases globally
        all_aliases: list[str] = []
        for r in refs:
            all_aliases.extend(r.aliases)

        alias_counts = Counter(all_aliases)
        if repeated := [a for a, c in alias_counts.items() if c > 1]:
            raise ValueError(f"Duplicate aliases: {repeated[:5]}")

        # Check collision between Names and Aliases
        collisions = set(names.keys()) & set(alias_counts.keys())
        if collisions:
            raise ValueError(f"Ambiguous identifiers (used as both Name and Alias): {list(collisions)[:5]}")

    def canonical_names(self) -> Mapping[str, str]:
        """Map string identifiers (names/aliases) to canonical names."""
        return PropertyMap(self._mapping, self._names)

    def lengths(self) -> Mapping[str, int]:
        """Map canonical names to sequence lengths."""
        return PropertyMap(self._index, self._lengths)

    def labels(self) -> Mapping[str, str]:
        """Map canonical names to human-readable labels."""
        return PropertyMap(self._index, self._labels)

    def descriptions(self) -> Mapping[str, str | None]:
        """Map canonical names to descriptions."""
        return PropertyMap(self._index, self._descriptions)

    def aliases(self) -> Mapping[str, frozenset[str]]:
        """Map canonical names to their alias sets."""
        return PropertyMap(self._index, self._aliases)

    def resolve(self, identifier: str) -> ReferenceProxy:
        idx = self._mapping.get(identifier)
        if idx is None:
            raise KeyError(f"Identifier '{identifier}' not found.")
        return ReferenceProxy(idx, self)

    def get(self, identifier: str, default: ReferenceProxy | None = None) -> ReferenceProxy | None:
        idx = self._index.get(identifier)
        if idx is None:
            return default
        return ReferenceProxy(idx, self)

    def references(self) -> Iterable[ReferenceProxy]:
        for idx in range(len(self)):
            yield ReferenceProxy(idx, self)

    @classmethod
    def merge(cls, registries: Iterable["RefRegistry"]) -> "RefRegistry":
        """
        Merge multiple RefRegistries into a new one using a Min-Heap.

        This performs a K-Way merge sort, which is efficient (O(N log K))
        and preserves the sorted order of canonical names.
        """
        # Materialize to allow indexing
        registries = tuple(registries)
        if not registries:
            raise ValueError("At least one RefRegistry must be provided for merging.")

        # Optimization for single registry
        if len(registries) == 1:
            return registries[0]

        # Initialize Structure of Arrays builders
        names: list[str] = []
        lengths: list[int] = []
        labels: list[str] = []
        descriptions: list[str | None] = []
        aliases: list[frozenset[str]] = []
        index, mapping = {}, {}

        # Heap stores tuples of: (Canonical Name, Registry Index)
        # We assume each input registry maintains sorted order of _names (guaranteed by __init__)
        heap: list[tuple[str, int]] = []
        cursors = [0] * len(registries)
        for i, registry in enumerate(registries):
            if len(registry) > 0:
                heapq.heappush(heap, (registry._names[0], i))

        # Iterate until the heap is empty
        while heap:
            # Pop the lexicographically smallest reference
            name, regidx = heapq.heappop(heap)

            # If the name was already processed, it means another registry
            # (or the same one) had a duplicate, which is invalid.
            if name in mapping:
                raise ValueError(f"Duplicate reference name '{name}' detected during merge.")
            registry, refidx = registries[regidx], cursors[regidx]

            # Append data for this reference
            index[name] = len(names)
            mapping[name] = len(names)

            ref_aliases = registry._aliases[refidx]
            aliases.append(ref_aliases)
            for alias in ref_aliases:
                if alias in mapping:
                    raise ValueError(f"Duplicate alias '{alias}' detected during merge.")
                mapping[alias] = len(names)

            names.append(name)
            lengths.append(registry._lengths[refidx])
            labels.append(registry._labels[refidx])
            descriptions.append(registry._descriptions[refidx])

            # Advance the cursor of the registry we just processed
            cursors[regidx] += 1
            if cursors[regidx] < len(registry):
                name = registry._names[cursors[regidx]]
                # Push the next available reference from this registry
                heapq.heappush(heap, (name, regidx))

        return cls._from_buffers(
            self=cls.__new__(cls),
            index=index,
            mapping=mapping,
            names=tuple(names),
            lengths=tuple(lengths),
            labels=tuple(labels),
            descriptions=tuple(descriptions),
            aliases=tuple(aliases)
        )

    def __iter__(self) -> Iterator[str]:
        """Iterate over canonical names."""
        return iter(self._names)

    def __getitem__(self, key: str) -> ReferenceProxy:
        item = self.get(key)
        if item is None:
            raise KeyError(f"Identifier '{key}' not found.")
        return item

    def __len__(self) -> int:
        return len(self._names)
