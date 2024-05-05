from enum import Enum

from attrs import define, field


class Stranding(Enum):
    Unstranded = "unstranded"
    Forward = "forward"
    Reverse = "reverse"
    Unknown = "unknown"

    def __repr__(self) -> str:
        return f"Stranding({self.value})"

    def __str__(self) -> str:
        return self.value

    @classmethod
    def normalize(cls, value: str) -> 'Stranding':
        match value.lower():
            case 'unstranded' | 'u':
                return cls.Unstranded
            case 'forward' | 'f':
                return cls.Forward
            case 'reverse' | 'r':
                return cls.Reverse
            case 'unknown' | 'x':
                return cls.Unknown
            case _:
                raise ValueError(f"Unknown stranding: {value}")


@define(slots=True, frozen=True, eq=True, order=True, repr=True, hash=True)
class Library:
    """
    A class to describe meta information about a sequencing library made from a biological sample.

    Attributes
    ----------
    source : tuple[str, ...]
        What molecules were used to generate the library?
    selection : tuple[str, ...]
        Were there any selection/enrichment steps during library generation?
    stranding : Stranding
        What is the stranding of the library?
    """
    source: tuple[str, ...] = field(converter=lambda x: tuple(x))
    selection: tuple[str, ...] = field(converter=lambda x: tuple(x))
    stranding: Stranding = field(converter=Stranding)

    @source.validator
    def check_source(self, _, value):
        if not value:
            raise ValueError("Library source must be specified")

    @selection.validator
    def check_selection(self, _, value):
        if not value:
            raise ValueError("Library selection method must be specified")
