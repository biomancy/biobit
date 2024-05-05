from attrs import define, field

from .orientation import Orientation
from .range import Range


@define(slots=True, eq=True)
class Interval:
    contig: str
    rng: Range
    orient: Orientation = field(default=Orientation.dual)
