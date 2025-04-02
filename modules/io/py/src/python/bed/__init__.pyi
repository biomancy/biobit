from pathlib import Path
from types import TracebackType
from typing import Iterable, Self

from biobit.core.loc import IntoInterval, Interval, IntoOrientation, Orientation
from biobit.io.protocols import ReadRecord, WriteRecord, IntoDecoding, IntoEncoding

type AnyBed = Bed3 | Bed4 | Bed5 | Bed6 | Bed8 | Bed9 | Bed12


class Bed3:
    seqid: str

    Reader: type[Reader[Bed3]]
    Writer: type[Writer[Bed3]]

    def __init__(self, seqid: str, interval: IntoInterval): ...

    @staticmethod
    def default() -> Bed3: ...

    @property
    def interval(self) -> Interval: ...

    @interval.setter
    def interval(self, interval: IntoInterval): ...

    def set(self, seqid: str | None = None, interval: IntoInterval | None = None): ...


class Bed4:
    seqid: str
    name: str

    Reader: type[Reader[Bed4]]
    Writer: type[Writer[Bed4]]

    def __init__(self, seqid: str, interval: IntoInterval, name: str): ...

    @staticmethod
    def default() -> Bed4: ...

    @property
    def interval(self) -> Interval: ...

    @interval.setter
    def interval(self, interval: IntoInterval): ...

    def set(self, seqid: str | None = None, interval: IntoInterval | None = None, name: str | None = None): ...


class Bed5:
    seqid: str
    name: str
    score: int

    Reader: type[Reader[Bed5]]
    Writer: type[Writer[Bed5]]

    def __init__(self, seqid: str, interval: IntoInterval, name: str, score: int): ...

    @staticmethod
    def default() -> Bed5: ...

    @property
    def interval(self) -> Interval: ...

    @interval.setter
    def interval(self, interval: IntoInterval): ...

    def set(
            self, seqid: str | None = None, interval: IntoInterval | None = None, name: str | None = None,
            score: int | None = None
    ): ...


class Bed6:
    seqid: str
    name: str
    score: int

    Reader: type[Reader[Bed6]]
    Writer: type[Writer[Bed6]]

    def __init__(self, seqid: str, interval: IntoInterval, name: str, score: int, orientation: IntoOrientation): ...

    @staticmethod
    def default() -> Bed6: ...

    @property
    def interval(self) -> Interval: ...

    @interval.setter
    def interval(self, interval: IntoInterval): ...

    @property
    def orientation(self) -> Orientation: ...

    @orientation.setter
    def orientation(self, orientation: IntoOrientation): ...

    def set(
            self, seqid: str | None = None, interval: IntoInterval | None = None, name: str | None = None,
            score: int | None = None, orientation: IntoOrientation | None = None
    ): ...


class Bed8:
    seqid: str
    name: str
    score: int

    Reader: type[Reader[Bed8]]
    Writer: type[Writer[Bed8]]

    def __init__(self, seqid: str, interval: IntoInterval, name: str, score: int, orientation: IntoOrientation,
                 thick: IntoInterval): ...

    @staticmethod
    def default() -> Bed8: ...

    @property
    def interval(self) -> Interval: ...

    @interval.setter
    def interval(self, interval: IntoInterval): ...

    @property
    def orientation(self) -> Orientation: ...

    @orientation.setter
    def orientation(self, orientation: IntoOrientation): ...

    @property
    def thick(self) -> Interval: ...

    @thick.setter
    def thick(self, thick: IntoInterval): ...

    def set(
            self, seqid: str | None = None, interval: IntoInterval | None = None, name: str | None = None,
            score: int | None = None, orientation: IntoOrientation | None = None, thick: IntoInterval | None = None
    ): ...


class Bed9:
    seqid: str
    name: str
    score: int
    rgb: tuple[int, int, int]

    Reader: type[Reader[Bed9]]
    Writer: type[Writer[Bed9]]

    def __init__(self, seqid: str, interval: IntoInterval, name: str, score: int, orientation: IntoOrientation,
                 thick: IntoInterval, rgb: tuple[int, int, int]): ...

    @staticmethod
    def default() -> Bed9: ...

    @property
    def interval(self) -> Interval: ...

    @interval.setter
    def interval(self, interval: IntoInterval): ...

    @property
    def orientation(self) -> Orientation: ...

    @orientation.setter
    def orientation(self, orientation: IntoOrientation): ...

    @property
    def thick(self) -> Interval: ...

    @thick.setter
    def thick(self, thick: IntoInterval): ...

    def set(
            self, seqid: str | None = None, interval: IntoInterval | None = None, name: str | None = None,
            score: int | None = None, orientation: IntoOrientation | None = None, thick: IntoInterval | None = None,
            rgb: tuple[int, int, int] | None = None
    ): ...


class Bed12:
    seqid: str
    name: str
    score: int
    rgb: tuple[int, int, int]

    Reader: type[Reader[Bed12]]
    Writer: type[Writer[Bed12]]

    def __init__(self, seqid: str, interval: IntoInterval, name: str, score: int, orientation: IntoOrientation,
                 thick: IntoInterval, rgb: tuple[int, int, int], blocks: list[IntoInterval]): ...

    @staticmethod
    def default() -> Bed12: ...

    @property
    def interval(self) -> Interval: ...

    @interval.setter
    def interval(self, interval: IntoInterval): ...

    @property
    def orientation(self) -> Orientation: ...

    @orientation.setter
    def orientation(self, orientation: IntoOrientation): ...

    @property
    def thick(self) -> Interval: ...

    @thick.setter
    def thick(self, thick: IntoInterval): ...

    @property
    def blocks(self) -> list[Interval]: ...

    @blocks.setter
    def blocks(self, blocks: list[IntoInterval]): ...

    def set(
            self, seqid: str | None = None, interval: IntoInterval | None = None, name: str | None = None,
            score: int | None = None, orientation: IntoOrientation | None = None, thick: IntoInterval | None = None,
            rgb: tuple[int, int, int] | None = None, blocks: list[IntoInterval] | None = None
    ): ...


class Reader[T](ReadRecord[T]):
    def __init__(self, path: str | Path, compression: IntoDecoding | None = None): ...

    @staticmethod
    def bed3(path: str | Path, compression: IntoDecoding | None = None) -> Reader[Bed3]: ...

    @staticmethod
    def bed4(path: str | Path, compression: IntoDecoding | None = None) -> Reader[Bed4]: ...

    @staticmethod
    def bed5(path: str | Path, compression: IntoDecoding | None = None) -> Reader[Bed5]: ...

    @staticmethod
    def bed6(path: str | Path, compression: IntoDecoding | None = None) -> Reader[Bed6]: ...

    @staticmethod
    def bed8(path: str | Path, compression: IntoDecoding | None = None) -> Reader[Bed8]: ...

    @staticmethod
    def bed9(path: str | Path, compression: IntoDecoding | None = None) -> Reader[Bed9]: ...

    @staticmethod
    def bed12(path: str | Path, compression: IntoDecoding | None = None) -> Reader[Bed12]: ...

    def read_record(self, into: T | None = None) -> T: ...

    def read_to_end(self) -> list[T]: ...

    def __iter__(self) -> Reader: ...

    def __next__(self) -> T: ...

    __hash__ = None  # type: ignore


class Writer[T](WriteRecord[T]):
    def __init__(self, path: str | Path, compression: IntoEncoding | None = None): ...

    @staticmethod
    def bed3(path: str | Path, compression: IntoEncoding | None = None) -> Writer[Bed3]: ...

    @staticmethod
    def bed4(path: str | Path, compression: IntoEncoding | None = None) -> Writer[Bed4]: ...

    @staticmethod
    def bed5(path: str | Path, compression: IntoEncoding | None = None) -> Writer[Bed5]: ...

    @staticmethod
    def bed6(path: str | Path, compression: IntoEncoding | None = None) -> Writer[Bed6]: ...

    @staticmethod
    def bed8(path: str | Path, compression: IntoEncoding | None = None) -> Writer[Bed8]: ...

    @staticmethod
    def bed9(path: str | Path, compression: IntoEncoding | None = None) -> Writer[Bed9]: ...

    @staticmethod
    def bed12(path: str | Path, compression: IntoEncoding | None = None) -> Writer[Bed12]: ...

    def write_record(self, record: T) -> Self: ...

    def write_records(self, records: Iterable[T]) -> Self: ...

    def flush(self) -> Self: ...

    def close(self) -> None: ...

    def __enter__(self) -> Writer: ...

    def __exit__(self, exc_type: type[BaseException] | None, exc_val: BaseException | None,
                 exc_tb: TracebackType | None): ...
