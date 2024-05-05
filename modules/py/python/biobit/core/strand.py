from enum import Enum
from typing import Literal, TypeVar, Generic

from attrs import define, field


class Strand(Enum):
    fwd = "+"
    rev = "-"

    def __repr__(self):
        return self.value

    def __str__(self):
        return self.value

    @classmethod
    def normalize(cls, strand: 'str | int | Strand') -> 'Strand':
        match strand:
            case "+" | 1:
                return Strand.fwd
            case "-" | -1:
                return Strand.rev
            case _ if isinstance(strand, cls):
                return strand
            case _:
                raise ValueError(f"Unknown strand: {strand}")


StrandLike = Strand | Literal["+", "-", 1, -1]

_T = TypeVar("_T")


@define(slots=True, frozen=True, eq=True, repr=True, hash=True)
class Stranded(Generic[_T]):
    fwd: _T = field()
    rev: _T = field()

    def with_rev(self, value: _T) -> 'Stranded[_T]':
        return Stranded(self.fwd, value)

    def with_fwd(self, value: _T) -> 'Stranded[_T]':
        return Stranded(value, self.rev)

    def __iter__(self):
        yield from (self.fwd, self.rev)
