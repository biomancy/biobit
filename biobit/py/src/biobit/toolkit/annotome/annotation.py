from abc import ABCMeta, abstractmethod
from pathlib import Path
from typing import Iterable


class Annotation(metaclass=ABCMeta):
    @abstractmethod
    def files(self) -> Iterable[Path]:
        ...
