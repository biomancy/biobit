from .. import repeats as repeats
from .filter import Filter
from .scoring import Scoring

def run(
    seq: bytes, filter: Filter, scoring: Scoring
) -> tuple[list[repeats.InvRepeat], list[int]]:
    """
    Predict inverted repeats in the given nucleic acid sequence.

    :param seq: ASCII DNA or RNA sequence containing A, C, G, U, T, or N (case insensitive).
        T is converted to U; other bytes are rejected.
    :param filter: filter applied to predicted inverted repeats
    :param scoring: alignment scoring schema
    :return: list of inverted repeats satisfying given constraints together with their alignment scores
    """

__all__ = ["run"]
