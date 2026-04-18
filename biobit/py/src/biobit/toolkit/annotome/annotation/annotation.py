from abc import ABCMeta, abstractmethod
from typing import Iterable, Self

from biobit.toolkit.annotome.reference import RefRegistry


class Annotation(metaclass=ABCMeta):
    """
    Abstract contract for data attached to an Assembly.
    Must support merging to allow Assembly merging.
    """

    @classmethod
    @abstractmethod
    def merge(cls, refrg: RefRegistry, items: Iterable[tuple[RefRegistry, Self | None]]) -> Self:
        """
        Merge multiple Annotation instances into one.
        """
        ...
