from .filter import Filter
from .scoring import Scoring
from .. import repeats as repeats


def run(seq: bytes, filter: Filter, scoring: Scoring) -> tuple[list[repeats.InvRepeat], list[int]]:
    """
    Predict inverted repeats in the given nucleic acid sequence.

    :param seq: raw ASCII string, DNA or RNA sequence
    :param filter: filter applied to predicted inverted repeats
    :param scoring: alignment scoring schema
    :return: list of inverted repeats satisfying given constraints together with their alignment scores
    """
    ...
