from collections.abc import Mapping
from typing import Hashable

from attr import define, field

from biobit.core.loc import Strand


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class Meta[AttrsDict: Mapping, TagValue: Hashable]:
    source: str
    attrs: AttrsDict
    tags: frozenset[TagValue] = field(factory=frozenset)


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class Location:
    # Coordinates
    seqid: str
    strand: Strand = field(converter=lambda x: Strand(x))
    start: int
    end: int
