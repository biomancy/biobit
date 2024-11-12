from collections.abc import Iterator, Sequence

from .interval import IntoInterval, Interval


class ChainInterval:
    def __init__(self, links: IntoChainInterval) -> None: ...

    def links(self) -> list[Interval]: ...

    def __iter__(self) -> Iterator[Interval]: ...


IntoChainInterval = ChainInterval | Sequence[IntoInterval]
