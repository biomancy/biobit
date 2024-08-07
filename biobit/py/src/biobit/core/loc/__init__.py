from typing import Literal

from biobit._biobit.core.loc import Strand, Orientation, Segment, Locus, PerOrientation

IntoOrientation = Orientation | Literal["+", "-", "=", 1, -1, 0]
IntoStrand = Strand | Literal["+", "-", 1, -1]
IntoSegment = Segment | tuple[int, int]

__all__ = [
    "Strand", "Orientation", "Segment", "Locus", "PerOrientation", "IntoOrientation", "IntoStrand", "IntoSegment"
]
