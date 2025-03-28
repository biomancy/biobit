from typing import Literal, ClassVar


class Strandedness:
    Forward: ClassVar[Strandedness]
    Reverse: ClassVar[Strandedness]
    Unstranded: ClassVar[Strandedness]

    def __init__(self, value: IntoStrandedness) -> None: ...

    def __repr__(self) -> str: ...

    def __str__(self) -> str: ...

    def __hash__(self) -> int: ...

    def __eq__(self, other: object) -> bool: ...

    def __ne__(self, other: object) -> bool: ...

    def __lt__(self, other: object) -> bool: ...

    def __le__(self, other: object) -> bool: ...

    def __gt__(self, other: object) -> bool: ...

    def __ge__(self, other: object) -> bool: ...


IntoStrandedness = Strandedness | Literal["F", "R", "U"]
