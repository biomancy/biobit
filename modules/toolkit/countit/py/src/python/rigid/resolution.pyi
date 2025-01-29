class AnyOverlap:
    def __init__(self) -> None: ...


class OverlapWeighted:
    def __init__(self) -> None: ...


class TopRanked[E]:
    def __init__(self, priority: list[E]) -> None: ...


type IntoResolution = AnyOverlap | OverlapWeighted | TopRanked
