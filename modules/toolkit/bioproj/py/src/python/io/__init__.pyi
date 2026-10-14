import os

from . import json as json

type StrPath = str | os.PathLike[str]

__all__ = ["StrPath", "json"]
