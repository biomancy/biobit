from collections.abc import Sequence
from typing import Literal

from biobit.rs.core.loc import Strand, Orientation, Interval, ChainInterval, PerOrientation, PerStrand

from . import mapping

IntoOrientation = Orientation | Literal["+", "-", "=", 1, -1, 0]
IntoStrand = Strand | Literal["+", "-", 1, -1]
IntoInterval = Interval | tuple[int, int]
IntoChainInterval = ChainInterval | Sequence[IntoInterval]

__all__ = [
    "Strand", "Orientation", "Interval", "ChainInterval", "PerOrientation", "PerStrand",
    "IntoOrientation", "IntoStrand", "IntoInterval", "IntoChainInterval", "mapping",
]
