from typing import Self, final

@final
class AnyOverlap:
    def __new__(cls) -> Self: ...

@final
class OverlapWeighted:
    def __new__(cls) -> Self: ...

@final
class TopRanked[E]:
    def __new__(cls, priority: list[E]) -> Self: ...

type IntoResolution = AnyOverlap | OverlapWeighted | TopRanked

__all__ = ["AnyOverlap", "IntoResolution", "OverlapWeighted", "TopRanked"]
