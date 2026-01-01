from collections.abc import Iterable
from pathlib import Path
from typing import Self

from biobit.io.fasta import IndexedReader
from .annotation import Annotation
from ..reference import RefRegistry


class Fasta(Annotation):
    __slots__ = ("files",)

    def __init__(self, files: Iterable[Path | str]) -> None:
        self.files = tuple(Path(f) for f in files)

    def reader(self) -> IndexedReader:
        return IndexedReader(self.files)

    @classmethod
    def merge(cls, refrg: RefRegistry, items: Iterable[tuple[RefRegistry, Self | None]]) -> Self:
        files = []
        for _, item in items:
            if item is None:
                raise ValueError("Fasta annotation should be provided for all references when merging.")
            files.extend(item.files)
        return Fasta(files)
