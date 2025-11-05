from abc import ABCMeta, abstractmethod
from typing import Iterable

from .annotation import Annotation
from .seqinfo import SeqInfo


class Assembly(metaclass=ABCMeta):
    """A formal contract for a genomic assembly."""

    @abstractmethod
    def name(self) -> str:
        """A unique name for the assembly (e.g., 'GRCh38')."""
        ...

    @abstractmethod
    def organisms(self) -> Iterable[str]:
        """Organisms covered by the assembly (e.g., 'Homo sapiens', 'HSV-1')."""
        ...

    @abstractmethod
    def seqinfo(self) -> SeqInfo:
        """Return information about sequences in this assembly"""
        ...

    @abstractmethod
    def annotations(self) -> Iterable[str]:
        """All annotations available for the assembly."""
        ...

    @abstractmethod
    def fetch(self, annotation: str) -> Annotation:
        """Fetch the annotation with the given key and type T."""
        ...
