from collections.abc import Iterable, Iterator
from types import TracebackType
from typing import Literal, Protocol, Self

# "infer" uses the filename suffix; None explicitly selects an uncompressed stream.
type Compression = Literal["infer", "gz", "bgz"] | None


class ReadRecord[T](Protocol):
    def read_record(self, into: T | None = None) -> T | None: ...

    def read_to_end(self) -> list[T]: ...

    def __iter__(self) -> Iterator[T]: ...


class WriteRecord[T](Protocol):
    def write_record(self, record: T): ...

    def write_records(self, records: Iterable[T]): ...

    def flush(self): ...

    def __enter__(self) -> Self: ...

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ): ...
