from attr import define, field

from biobit.core.loc import Strand
from .core import Location, Bundle, Entry


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class Gene[Attrs](Entry[Attrs]):
    ind: str
    loc: Location
    attrs: Attrs

    transcripts: frozenset[str] = field(converter=lambda x: frozenset(x))

    @property
    def tss(self) -> int:
        return self.loc.start if self.loc.strand == Strand.Forward else self.loc.end

    @property
    def tes(self) -> int:
        return self.loc.end if self.loc.strand == Strand.Forward else self.loc.start


class GeneBundle[Attrs](Bundle[Gene[Attrs]]):
    ...
