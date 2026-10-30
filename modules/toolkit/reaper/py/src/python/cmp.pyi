from typing import Self, final

@final
class Enrichment:
    """Fold enrichment with a pseudocount of one in normalized count units."""

    def __new__(cls) -> Self: ...
    def set_pseudocount(self, pseudocount: float) -> Enrichment: ...

__all__ = ["Enrichment"]
