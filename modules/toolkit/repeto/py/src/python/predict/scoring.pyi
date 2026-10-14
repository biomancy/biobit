from typing import final

@final
class Scoring:
    gap_open: int
    gap_extend: int

    complementary: int
    mismatch: int

__all__ = ["Scoring"]
