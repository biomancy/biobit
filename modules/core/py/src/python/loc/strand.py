from typing import Literal

from biobit.rs.core.loc import Strand

IntoStrand = Strand | Literal["+", "-", 1, -1]

__all__ = ["IntoStrand", "Strand"]
