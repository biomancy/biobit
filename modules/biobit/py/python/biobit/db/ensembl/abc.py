from abc import ABC
from enum import Enum
from pathlib import Path
from typing import Self


class EnsemblAttribute(Enum):
    @classmethod
    def fetch(
            cls,
            attributes: set[Self],
            organism: str,
            saveto: Path,
            force: bool,
            url: str,
            verbose: bool,
    ):
        raise ValueError("Not implemented")

    @property
    def colname(self) -> str:
        raise ValueError("Not implemented")

    @property
    def dtype(self):
        raise ValueError("Not implemented")


class EnsemblDescriptor(ABC):
    def _getter(self, mapping, id):
        if id not in mapping:
            id = id.split(".")[0]
        return mapping.get(id, None)
