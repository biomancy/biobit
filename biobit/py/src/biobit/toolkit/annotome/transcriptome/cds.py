from attr import define, field

from biobit.core.loc import Interval
from .core import Location, Bundle, Entry


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class CDS[Attrs](Entry[Attrs]):
    ind: str
    loc: Location
    attrs: Attrs

    blocks: tuple[Interval, ...] = field(converter=lambda x: tuple(x))


class CDSBundle[Attrs](Bundle[CDS[Attrs]]):
    ...
