from collections.abc import Mapping
from typing import Hashable, Iterable

from attr import define, field

from biobit.core.loc import Segment, Strand
from .core import Location, Meta


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class RNA[AttrsDict: Mapping, TagValue: Hashable]:
    id: str
    gene: str

    loc: Location
    meta: Meta[AttrsDict, TagValue]
    exons: tuple[Segment, ...] = field(converter=lambda x: tuple(x))

    @property
    def introns(self) -> tuple[Segment, ...]:
        introns = []
        for i in range(1, len(self.exons)):
            introns.append(Segment(self.exons[i - 1].end, self.exons[i].start))
        return tuple(introns)

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
class RNABundle[AttrsDict: Mapping, TagValue: Hashable](Mapping[str, RNA[AttrsDict, TagValue]]):
    all_attrs: frozenset[str] = field(init=False)
    all_tags: frozenset[TagValue] = field(init=False)
    _index: dict[str, RNA[AttrsDict, TagValue]] = field(init=False)

    def __init__(self, items: Iterable[RNA[AttrsDict, TagValue]]):
        index: dict[str, RNA[AttrsDict, TagValue]] = {}
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

    def __getitem__(self, __key) -> RNA[AttrsDict, TagValue]:
        return self._index[__key]

    def __len__(self) -> int:
        return len(self._index)

    def __contains__(self, __key) -> bool:
        return __key in self._index

    def __iter__(self):
        return iter(self._index.keys())
