from enum import Enum
from typing import Literal, Union

from .strand import Strand


class Orientation(Enum):
    fwd = "+"
    rev = "-"
    dual = "."

    def __repr__(self):
        return self.value

    def __str__(self):
        return self.value

    @classmethod
    def normalize(cls, orientation: 'OrientationLike') -> 'Orientation':
        match orientation:
            case "+" | 1:
                return Orientation.fwd
            case "-" | -1:
                return Orientation.rev
            case "." | 0:
                return Orientation.dual
            case _ if isinstance(orientation, cls):
                return orientation
            case _ if isinstance(orientation, Strand):
                return cls.from_strand(orientation)
            case _:
                raise ValueError(f"Unknown strand: {orientation}")

    @classmethod
    def from_strand(cls, strand: Strand) -> 'Orientation':
        match strand:
            case Strand.fwd:
                return cls.fwd
            case Strand.rev:
                return cls.rev
            case _:
                raise ValueError(f"Unknown strand: {strand}")


OrientationLike = Union[Strand, Orientation, Literal["+", "-", ".", 1, -1, 0]]
