from collections.abc import Iterable

from biobit.core.loc import IntoInterval, IntoOrientation

class Mismatches:
    minmismatches: int
    minfreq: float
    mincov: int

    def __init__(self, minmismatches: int = 1, minfreq: float = 0.0, mincov: int = 1) -> None: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore

class RequiredSites:
    len: int

    def __init__(
        self,
        required: Iterable[tuple[str, IntoOrientation, Iterable[IntoInterval]]] | None = None,
    ) -> None: ...
    def is_empty(self) -> bool: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore

class RequiredOrMismatches:
    required: RequiredSites
    mismatches: Mismatches

    def __init__(self, required: RequiredSites | None = None, mismatches: Mismatches | None = None) -> None: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore
