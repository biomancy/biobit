from pathlib import Path

from biobit.core.loc import IntoInterval
from biobit.io.protocols import ReadRecord


class Record:
    id: str
    seq: str

    def __init__(self, id: str, seq: str): ...


class Reader(ReadRecord[Record]):
    def __init__(self, path: str | Path): ...

    def read_record(self, into: Record | None = None) -> Record: ...

    def read_to_end(self) -> list[Record]: ...

    def __iter__(self) -> Reader: ...

    def __next__(self) -> Record: ...

    __hash__ = None  # type: ignore


class IndexedReader:
    def __init__(self, path: str | Path): ...

    @property
    def path(self) -> Path: ...

    def fetch(self, seqid: str, interval: IntoInterval) -> str: ...

    def fetch_full_seq(self, seqid: str) -> str: ...

    __hash__ = None  # type: ignore
