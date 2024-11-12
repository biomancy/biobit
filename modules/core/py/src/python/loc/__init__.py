from typing import Literal

from biobit._biobit.core.loc import Strand, Orientation, Interval, Locus, PerOrientation, PerStrand

IntoOrientation = Orientation | Literal["+", "-", "=", 1, -1, 0]
IntoStrand = Strand | Literal["+", "-", 1, -1]
IntoInterval = Interval | tuple[int, int]
IntoLocus = Locus | tuple[str, IntoInterval, IntoOrientation]

__all__ = [
    "Strand", "Orientation", "Interval", "Locus", "PerOrientation", "PerStrand",
    "IntoOrientation", "IntoStrand", "IntoInterval", "IntoLocus"
]
