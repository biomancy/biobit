from typing import Iterator

from biobit.core.loc import Segment, IntoSegment


class Elements[T]:
    def __init__(self) -> None: ...

    @staticmethod
    def from_existent(segments: list[list[IntoSegment]], elements: list[list[T]]) -> 'Elements[T]': ...

    @property
    def segments(self) -> list[list[Segment]]: ...

    @property
    def elements(self) -> list[list[T]]: ...

    def __iter__(self) -> Iterator[tuple[list[Segment], list[T]]]: ...

    def __len__(self) -> int: ...


class Steps[T]:
    def __init__(self) -> None: ...

    def build(self, elements: Elements[T], query: list[IntoSegment]) -> None: ...

    def __iter__(self) -> Iterator[tuple[IntoSegment, set[T]]]: ...

    def __len__(self) -> int: ...
