from typing import Literal

from biobit.rs.core.loc import Orientation

IntoOrientation = Orientation | Literal["+", "-", "=", 1, -1, 0]

__all__ = ["IntoOrientation", "Orientation"]
