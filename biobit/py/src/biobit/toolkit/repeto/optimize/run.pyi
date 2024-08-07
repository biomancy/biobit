from .. import repeats as repeats


def run(ir: list[repeats.InvRepeat], scores: list[int]) -> tuple[list[repeats.InvRepeat], int]:
    """
    Find score-maximal and coherent set of inverted nucleic acid repeats.

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

    :param ir: list of InvertedRepeat objects
    :param scores: numerical score for each InvertedRepeat
    :return: Tuple containing an optimal set of inverted repeats and the associated total score
    """
    ...
