from typing import Self, final

@final
class Enrichment:
    def __new__(cls) -> Self: ...
    def set_scaling(self, signal: float, control: float) -> Enrichment: ...

__all__ = ["Enrichment"]
