from collections.abc import Sequence
from typing import Literal

from . import mapping

from biobit._biobit.core.loc import Strand, Orientation, Interval, ChainInterval, Locus, PerOrientation, PerStrand

IntoOrientation = Orientation | Literal["+", "-", "=", 1, -1, 0]
IntoStrand = Strand | Literal["+", "-", 1, -1]
IntoInterval = Interval | tuple[int, int]
IntoLocus = Locus | tuple[str, IntoInterval, IntoOrientation]
IntoChainInterval = ChainInterval | Sequence[IntoInterval]

__all__ = [
    "Strand", "Orientation", "Interval", "ChainInterval", "Locus", "PerOrientation", "PerStrand",
    "IntoOrientation", "IntoStrand", "IntoInterval", "IntoLocus", "IntoChainInterval",
    "mapping",
]
