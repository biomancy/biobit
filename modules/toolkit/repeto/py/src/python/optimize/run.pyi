from .. import repeats as repeats

def run(
    ir: list[repeats.InvRepeat], scores: list[int]
) -> tuple[list[repeats.InvRepeat], int]:
    """
    Maximize the total score over repeat combinations supported by interval dynamic programming.

    Here, coherent matching means that there are no "interfering" inverted repeats in the final set.
    That is, there are no i and j where:
        left(i) <= left(j) < right(i) <= right(j)
    Visually (- is gap, * is matched nucleotide):
        *****--------*****
               ****---------****

    The following combinations are allowed:
        left(i) < right(i) < left(j) < right(j)
        *****--------*****    ****---------****

        left(i) < left(j) < right(j) < right(i)
        *****----------------------*****
               ****---------****

    Such coherent matching also represents a formally valid (but very rough) RNA secondary structure.

    Known limitation:
        Repeats can be combined only if they are disjoint or one fits entirely inside a
        single gap of the other. Linked repeats with interleaved segments may have compatible
        base pairs but still cannot be selected together. The result can therefore miss a
        higher-scoring valid combination.

    :param ir: list of InvertedRepeat objects. Each selected or excluded as a single unit.
    :param scores: numerical score for each InvertedRepeat
    :return: Selected repeats and their total score
    """

__all__ = ["run"]
