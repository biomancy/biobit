from attr import define, field

from biobit.core.loc import Segment, Strand
from .core import Location, Bundle, Entry


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class RNA[Attrs](Entry[Attrs]):
    ind: str
    loc: Location
    attrs: Attrs

    gene: str
    exons: tuple[Segment, ...] = field(converter=lambda x: tuple(x))

    @property
    def introns(self) -> tuple[Segment, ...]:
        introns = []
        for i in range(1, len(self.exons)):
            introns.append(Segment(self.exons[i - 1].end, self.exons[i].start))
        return tuple(introns)

    @property
    def tss(self) -> int:
        return self.loc.start if self.loc.strand == Strand.Forward else self.loc.end

    @property
    def tes(self) -> int:
        return self.loc.end if self.loc.strand == Strand.Forward else self.loc.start


class RNABundle[Attrs](Bundle[RNA[Attrs]]):
    ...
