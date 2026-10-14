from collections.abc import Sequence

from biobit.rs.core.loc import ChainInterval

from .interval import IntoInterval

IntoChainInterval = ChainInterval | Sequence[IntoInterval]

__all__ = ["ChainInterval", "IntoChainInterval"]
