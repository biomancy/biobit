from typing import Self, final

from biobit.core.loc.chain_interval import ChainInterval, IntoChainInterval
from biobit.core.loc.interval import Interval, IntoInterval

@final
class ChainMap:
    def __new__(cls, chain: IntoChainInterval) -> Self: ...
    def invmap_interval(self, interval: IntoInterval) -> ChainInterval | None: ...
    def map_interval(self, interval: IntoInterval) -> Interval | None: ...

__all__ = ["ChainMap"]
