from biobit.core.loc.chain_interval import IntoChainInterval, ChainInterval
from biobit.core.loc.interval import IntoInterval, Interval


class ChainMap:
    def __init__(self, chain: IntoChainInterval): ...

    def invmap_interval(self, interval: IntoInterval) -> ChainInterval | None: ...

    def map_interval(self, interval: IntoInterval) -> Interval | None: ...
