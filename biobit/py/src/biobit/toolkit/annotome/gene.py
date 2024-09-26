from collections.abc import Mapping, Iterable
from typing import Hashable

from attr import define, field

from biobit.core.loc import Strand
from .core import Meta, Location


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class Gene[AttrsDict: Mapping, TagValue: Hashable]:
    id: str
    loc: Location
    meta: Meta[AttrsDict, TagValue]
    transcripts: frozenset[str] = field(converter=lambda x: frozenset(x))

    @property
    def tss(self) -> int:
        match self.loc.strand:
            case Strand.Forward:
                return self.loc.start
            case Strand.Reverse:
                return self.loc.end
            case _:
                raise ValueError(f"Invalid strand: {self.loc.strand}")

    @property
    def tes(self) -> int:
        match self.loc.strand:
            case Strand.Forward:
                return self.loc.end
            case Strand.Reverse:
                return self.loc.start
            case _:
                raise ValueError(f"Invalid strand: {self.loc.strand}")


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True, init=False)
class GeneBundle[AttrsDict: Mapping, TagValue: Hashable](Mapping[str, Gene[AttrsDict, TagValue]]):
    all_attrs: frozenset[str] = field(init=False)
    all_tags: frozenset[TagValue] = field(init=False)
    _index: dict[str, Gene[AttrsDict, TagValue]] = field(init=False)

    def __init__(self, items: Iterable[Gene[AttrsDict, TagValue]]):
        index: dict[str, Gene[AttrsDict, TagValue]] = {}
        attrs: set[str] = set()
        tags: set[TagValue] = set()
        for item in items:
            if item.id in index:
                raise ValueError(f"Duplicate RNA ID: {item.id}")
            index[item.id] = item
            attrs |= item.meta.attrs.keys()
            tags |= item.meta.tags
        object.__setattr__(self, '_index', index)
        object.__setattr__(self, 'all_attrs', frozenset(attrs))
        object.__setattr__(self, 'all_tags', frozenset(tags))

    def __getitem__(self, __key) -> Gene[AttrsDict, TagValue]:
        return self._index[__key]

    def __len__(self) -> int:
        return len(self._index)

    def __contains__(self, __key) -> bool:
        return __key in self._index

    def __iter__(self):
        return iter(self._index.keys())
