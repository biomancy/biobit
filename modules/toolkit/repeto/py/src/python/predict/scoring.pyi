from typing import final

@final
class Scoring:
    """Gap scores must satisfy ``gap_open <= gap_extend <= 0`` when predicting.

    A gap of length n scores ``gap_open + (n - 1) * gap_extend``.
    """

    gap_open: int
    gap_extend: int

    complementary: int
    mismatch: int

__all__ = ["Scoring"]
