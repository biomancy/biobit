from collections.abc import Iterable
from pathlib import Path
from types import TracebackType
from typing import Self, final, type_check_only

from biobit.core.loc import Interval, IntoInterval, IntoOrientation, Orientation
from biobit.io.protocols import IntoDecoding, IntoEncoding, ReadRecord, WriteRecord

type AnyBed = Bed3 | Bed4 | Bed5 | Bed6 | Bed8 | Bed9 | Bed12

@final
class Bed3:
    seqid: str

    Reader: type[_Bed3Reader]
    Writer: type[_Bed3Writer]

    def __new__(cls, seqid: str, interval: IntoInterval) -> Self: ...
    @staticmethod
    def default() -> Bed3: ...
    @property
    def interval(self) -> Interval: ...
    @interval.setter
    def interval(self, interval: IntoInterval): ...
    def set(self, seqid: str | None = None, interval: IntoInterval | None = None): ...

@final
class Bed4:
    seqid: str
    name: str

    Reader: type[_Bed4Reader]
    Writer: type[_Bed4Writer]

    def __new__(cls, seqid: str, interval: IntoInterval, name: str) -> Self: ...
    @staticmethod
    def default() -> Bed4: ...
    @property
    def interval(self) -> Interval: ...
    @interval.setter
    def interval(self, interval: IntoInterval): ...
    def set(
        self,
        seqid: str | None = None,
        interval: IntoInterval | None = None,
        name: str | None = None,
    ): ...

@final
class Bed5:
    seqid: str
    name: str
    score: int

    Reader: type[_Bed5Reader]
    Writer: type[_Bed5Writer]

    def __new__(
        cls, seqid: str, interval: IntoInterval, name: str, score: int
    ) -> Self: ...
    @staticmethod
    def default() -> Bed5: ...
    @property
    def interval(self) -> Interval: ...
    @interval.setter
    def interval(self, interval: IntoInterval): ...
    def set(
        self,
        seqid: str | None = None,
        interval: IntoInterval | None = None,
        name: str | None = None,
        score: int | None = None,
    ): ...

@final
class Bed6:
    seqid: str
    name: str
    score: int

    Reader: type[_Bed6Reader]
    Writer: type[_Bed6Writer]

    def __new__(
        cls,
        seqid: str,
        interval: IntoInterval,
        name: str,
        score: int,
        orientation: IntoOrientation,
    ) -> Self: ...
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
        self,
        seqid: str | None = None,
        interval: IntoInterval | None = None,
        name: str | None = None,
        score: int | None = None,
        orientation: IntoOrientation | None = None,
    ): ...

@final
class Bed8:
    seqid: str
    name: str
    score: int

    Reader: type[_Bed8Reader]
    Writer: type[_Bed8Writer]

    def __new__(
        cls,
        seqid: str,
        interval: IntoInterval,
        name: str,
        score: int,
        orientation: IntoOrientation,
        thick: IntoInterval,
    ) -> Self: ...
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
        self,
        seqid: str | None = None,
        interval: IntoInterval | None = None,
        name: str | None = None,
        score: int | None = None,
        orientation: IntoOrientation | None = None,
        thick: IntoInterval | None = None,
    ): ...

@final
class Bed9:
    seqid: str
    name: str
    score: int
    rgb: tuple[int, int, int]

    Reader: type[_Bed9Reader]
    Writer: type[_Bed9Writer]

    def __new__(
        cls,
        seqid: str,
        interval: IntoInterval,
        name: str,
        score: int,
        orientation: IntoOrientation,
        thick: IntoInterval,
        rgb: tuple[int, int, int],
    ) -> Self: ...
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
        self,
        seqid: str | None = None,
        interval: IntoInterval | None = None,
        name: str | None = None,
        score: int | None = None,
        orientation: IntoOrientation | None = None,
        thick: IntoInterval | None = None,
        rgb: tuple[int, int, int] | None = None,
    ): ...

@final
class Bed12:
    seqid: str
    name: str
    score: int
    rgb: tuple[int, int, int]

    Reader: type[_Bed12Reader]
    Writer: type[_Bed12Writer]

    def __new__(
        cls,
        seqid: str,
        interval: IntoInterval,
        name: str,
        score: int,
        orientation: IntoOrientation,
        thick: IntoInterval,
        rgb: tuple[int, int, int],
        blocks: list[IntoInterval],
    ) -> Self: ...
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
        self,
        seqid: str | None = None,
        interval: IntoInterval | None = None,
        name: str | None = None,
        score: int | None = None,
        orientation: IntoOrientation | None = None,
        thick: IntoInterval | None = None,
        rgb: tuple[int, int, int] | None = None,
        blocks: list[IntoInterval] | None = None,
    ): ...

# Shared structural contracts for the concrete Rust stream classes.
@type_check_only
class _Reader[T](ReadRecord[T]):
    def __new__(
        cls, path: str | Path, compression: IntoDecoding | None = None
    ) -> Self: ...
    def read_record(self, into: T | None = None) -> T | None: ...
    def read_to_end(self) -> list[T]: ...
    def __iter__(self) -> Self: ...
    def __next__(self) -> T: ...
    __hash__: None = None  # type: ignore[assignment]

@type_check_only
class _Writer[T](WriteRecord[T]):
    def __new__(
        cls, path: str | Path, compression: IntoEncoding | None = None
    ) -> Self: ...
    def write_record(self, record: T) -> Self: ...
    def write_records(self, records: Iterable[T]) -> Self: ...
    def flush(self) -> Self: ...
    def close(self) -> None: ...
    def __enter__(self) -> Self: ...
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
        /,
    ) -> None: ...

@final
class _Bed3Reader(_Reader[Bed3]): ...

@final
class _Bed3Writer(_Writer[Bed3]): ...

@final
class _Bed4Reader(_Reader[Bed4]): ...

@final
class _Bed4Writer(_Writer[Bed4]): ...

@final
class _Bed5Reader(_Reader[Bed5]): ...

@final
class _Bed5Writer(_Writer[Bed5]): ...

@final
class _Bed6Reader(_Reader[Bed6]): ...

@final
class _Bed6Writer(_Writer[Bed6]): ...

@final
class _Bed8Reader(_Reader[Bed8]): ...

@final
class _Bed8Writer(_Writer[Bed8]): ...

@final
class _Bed9Reader(_Reader[Bed9]): ...

@final
class _Bed9Writer(_Writer[Bed9]): ...

@final
class _Bed12Reader(_Reader[Bed12]): ...

@final
class _Bed12Writer(_Writer[Bed12]): ...

@final
class Reader:
    """Factories for the concrete BED stream types."""
    @staticmethod
    def bed3(
        path: str | Path, compression: IntoDecoding | None = None
    ) -> _Bed3Reader: ...
    @staticmethod
    def bed4(
        path: str | Path, compression: IntoDecoding | None = None
    ) -> _Bed4Reader: ...
    @staticmethod
    def bed5(
        path: str | Path, compression: IntoDecoding | None = None
    ) -> _Bed5Reader: ...
    @staticmethod
    def bed6(
        path: str | Path, compression: IntoDecoding | None = None
    ) -> _Bed6Reader: ...
    @staticmethod
    def bed8(
        path: str | Path, compression: IntoDecoding | None = None
    ) -> _Bed8Reader: ...
    @staticmethod
    def bed9(
        path: str | Path, compression: IntoDecoding | None = None
    ) -> _Bed9Reader: ...
    @staticmethod
    def bed12(
        path: str | Path, compression: IntoDecoding | None = None
    ) -> _Bed12Reader: ...

@final
class Writer:
    """Factories for the concrete BED stream types."""
    @staticmethod
    def bed3(
        path: str | Path, compression: IntoEncoding | None = None
    ) -> _Bed3Writer: ...
    @staticmethod
    def bed4(
        path: str | Path, compression: IntoEncoding | None = None
    ) -> _Bed4Writer: ...
    @staticmethod
    def bed5(
        path: str | Path, compression: IntoEncoding | None = None
    ) -> _Bed5Writer: ...
    @staticmethod
    def bed6(
        path: str | Path, compression: IntoEncoding | None = None
    ) -> _Bed6Writer: ...
    @staticmethod
    def bed8(
        path: str | Path, compression: IntoEncoding | None = None
    ) -> _Bed8Writer: ...
    @staticmethod
    def bed9(
        path: str | Path, compression: IntoEncoding | None = None
    ) -> _Bed9Writer: ...
    @staticmethod
    def bed12(
        path: str | Path, compression: IntoEncoding | None = None
    ) -> _Bed12Writer: ...

__all__ = [
    "AnyBed",
    "Bed3",
    "Bed4",
    "Bed5",
    "Bed6",
    "Bed8",
    "Bed9",
    "Bed12",
    "Reader",
    "Writer",
]
