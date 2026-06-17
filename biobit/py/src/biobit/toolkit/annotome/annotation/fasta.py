from collections.abc import Iterable
from pathlib import Path
from typing import Self

from biobit.io.fasta import IndexedReader, IndexedSources

from ..reference import RefRegistry
from .annotation import Annotation


class Fasta(Annotation):
    __slots__ = ("files",)

    def __init__(self, files: Iterable[Path | str] | Path | str) -> None:
        self.files = (
            (Path(files),)
            if isinstance(files, (str, Path))
            else tuple(Path(f) for f in files)
        )

    def reader(self) -> IndexedReader:
        return IndexedSources(self.files).open()

    @classmethod
    def merge(
        cls, refrg: RefRegistry, items: Iterable[tuple[RefRegistry, Self | None]]
    ) -> Self:
        files: list[Path] = []
        for _, item in items:
            if item is None:
                raise ValueError(
                    "Fasta annotation should be provided for all references when merging."
                )
            files.extend(item.files)
        return cls(files)
