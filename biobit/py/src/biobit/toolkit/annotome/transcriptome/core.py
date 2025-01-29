from typing import Protocol, Iterable

from attr import define, field

from biobit.core.loc import Strand


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class Location:
    # Coordinates
    seqid: str
    strand: Strand = field(converter=lambda x: Strand(x))
    start: int
    end: int

    @property
    def tss(self) -> int:
        return self.start if self.strand == Strand.Forward else self.end

    @property
    def tes(self) -> int:
        return self.end if self.strand == Strand.Forward else self.start


class Entry[Attrs](Protocol):
    ind: str
    loc: Location
    attrs: Attrs


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True, init=False)
class Bundle[T: Entry](dict[str, T]):
    def __init__(self, items: Iterable[T]):
        super().__init__()

        for item in items:
            if item.ind in self:
                raise ValueError(f"Duplicate ID: {item.ind}")
            self[item.ind] = item
