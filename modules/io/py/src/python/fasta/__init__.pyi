from collections.abc import Iterable
from pathlib import Path
from types import TracebackType
from typing import Self, final

from biobit.core.loc import IntoInterval
from biobit.io.protocols import IntoDecoding, IntoEncoding, ReadRecord, WriteRecord

@final
class Record:
    id: str
    seq: str

    def __new__(cls, header: str, sequence: str) -> Self: ...

@final
class Reader(ReadRecord[Record]):
    def __new__(
        cls, path: str | Path, compression: IntoDecoding | None = None
    ) -> Self: ...
    def read_record(self, into: Record | None = None) -> Record | None: ...
    def read_to_end(self) -> list[Record]: ...
    def __iter__(self) -> Reader: ...
    def __next__(self) -> Record: ...

    __hash__: None = None  # type: ignore[assignment]

@final
class IndexedReader:
    def fetch(self, seqid: str, interval: IntoInterval) -> str: ...
    def fetch_full_seq(self, seqid: str) -> str: ...
    def lengths(self) -> dict[str, int]: ...

    __hash__: None = None  # type: ignore[assignment]

@final
class IndexedSources:
    def __new__(cls, path: str | Path | Iterable[str | Path]) -> Self: ...
    def open(self) -> IndexedReader: ...

    __hash__: None = None  # type: ignore[assignment]

@final
class Writer(WriteRecord[Record]):
    def __new__(
        cls,
        path: str | Path,
        line_width: int | None = None,
        compression: IntoEncoding | None = None,
    ) -> Self: ...
    def write_record(self, record: Record) -> Self: ...
    def write_records(self, records: Iterable[Record]) -> Self: ...
    def flush(self) -> Self: ...
    def close(self) -> None: ...
    def __enter__(self) -> Writer: ...
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ): ...

__all__ = ["IndexedReader", "IndexedSources", "Reader", "Record", "Writer"]
