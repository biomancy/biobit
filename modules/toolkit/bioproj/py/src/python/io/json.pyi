from .. import Project
from . import StrPath

def read(path: StrPath) -> Project: ...
def write(path: StrPath, project: Project, pretty: bool = True) -> None: ...

__all__ = ["read", "write"]
