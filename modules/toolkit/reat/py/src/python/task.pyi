from biobit.core.loc import Interval, IntoInterval

class Task:
    seqid: str
    envelope: Interval
    intervals: list[Interval]

    def __init__(self, seqid: str, intervals: list[IntoInterval]) -> None: ...
    @staticmethod
    def from_intervals(
        intervals: list[tuple[str, IntoInterval]], max_task_size: int
    ) -> list[Task]: ...
    def __eq__(self, other: object) -> bool: ...

    __hash__ = None  # type: ignore
