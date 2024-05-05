import importlib.metadata
from pathlib import Path

CACHE = Path.home() / ".cache" / "biobit" / importlib.metadata.version('biobit')
