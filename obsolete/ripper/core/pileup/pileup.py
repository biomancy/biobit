import unittest
from copy import deepcopy
from dataclasses import dataclass
from typing import List, Tuple

import numba
import numpy as np
import numpy.typing as npt
from numba import float32, int32

from ..seqblocks import AlignedBlocks


@dataclass
class Pileup:
    # Genomic contig / Transcript id
    id: str
    # Pileup intervals are half-open[) and are represented by two arrays.
    # interend - end positions for each interval. Start positions for the intervals sequence is 0.
    # values - pileup values. I.e. normalized number of fragments covering a given region
    interend: npt.NDArray[np.int32]
    values: npt.NDArray[np.float32]

    def __post_init__(self):
        assert self.interend.dtype == np.int32 and self.values.dtype == np.float32
        self.owned()

    def owned(self) -> "Pileup":
        # A costly workaround to avoid creating arrays outside the numba function
        # In short, if array was subsampled in numba, then we can't force numpy to change it later
        # Unless the memory is owned, of course. Which can be easily done by copying the data
        try:
            self.values.setflags(write=True)
            self.interend.setflags(write=True)
        except ValueError:
            self.values = self.values.copy()
            self.interend = self.interend.copy()
        return self

    @staticmethod
    def from_tuples(contig: str, dense: List[Tuple[int, float]]) -> "Pileup":
        interend, values = [], []
        for end, v in dense:
            interend.append(end)
            values.append(v)
        return Pileup(
            contig,
            np.asarray(interend, dtype=np.int32),
            np.asarray(values, dtype=np.float32),
        )

    @staticmethod
    def constant(contig: str, contiglen: np.int32, value: np.float32):
        return Pileup(
            contig,
            np.asarray([contiglen], dtype=np.int32),
            np.asarray([value], dtype=np.float32),
        )


# Tuple((int32[:], float32[:]))(float32[:], float32, int32)
@numba.jit(cache=True, nopython=True, nogil=True)
def _simplify(dense_pileup, tolerance, maxbreakpoints):
    pos = 0
    interend = np.empty(2 * maxbreakpoints + 1, dtype=np.int32)
    values = np.empty(2 * maxbreakpoints + 1, dtype=np.float32)

    curval = dense_pileup[0]
    for i in range(1, len(dense_pileup)):
        if abs(dense_pileup[i] - curval) < tolerance:
            continue
        # Save current interval
        values[pos] = curval
        interend[pos] = i
        pos += 1
        # assert pos <= interend.size  # TODO comment later
        # Next value
        curval = dense_pileup[i]

    # Final interval
    interend[pos] = dense_pileup.size
    values[pos] = curval
    pos += 1

    interend, values = interend[:pos], values[:pos]
    return interend, values


# void(int32[:], int32[:], int32[:], int32, int32, float32[:])
@numba.jit(cache=True, nopython=True, nogil=True)
def _pileup(blstart, blend, index, extension, saveto):
    contiglen = saveto.size

    for i in range(len(index) - 1):
        readstart, readend = index[i], index[i + 1]
        # Single block
        if readend - readstart == 1:
            start, end = blstart[readstart], blend[readstart]
            length = end - start
            start, end = max(0, start - extension), min(contiglen, end + extension)
            newlen = end - start
            saveto[start:end] += length / newlen
        # 2 or more blocks
        else:
            starts, ends = blstart[readstart:readend], blend[readstart:readend]
            length = 0
            for j in range(readend - readstart):
                length += ends[j] - starts[j]

            # Caluclate new, extended ends
            extstart, extend = (
                max(0, starts[0] - extension),
                min(contiglen, ends[-1] + extension),
            )
            # New weighted value
            newlen = length + (starts[0] - extstart) + (extend - ends[-1])
            assert 0 < newlen <= length + 2 * extension
            value = length / newlen
            # Add to extended tails
            saveto[extstart: starts[0]] += value
            saveto[ends[-1]: extend] += value
            # Add to the body of the fragment
            for j in range(readend - readstart):
                saveto[starts[j]: ends[j]] += value
    return saveto


def calculate(
        contig: str,
        contiglen: np.int32,
        blocks: List[AlignedBlocks],
        extension: int32,
        sensitivity: float32 = float32(1e-5),
) -> Pileup:
    """
    :param blocks: Aligned blocks for each read on a contig. List to pool multiple BAM files in-memory
    :param extension: extend each read(only 5` and 3` terminal blocks) by a given value and re-normalize accordingly
    :param sensitivity: interval-wise pileup values will be insensitive to small changes in the original dense track

    Compute interval-wise pileup values, see Pileup for output format details.
    Each pileup value is max(baseline, sum(read blocks len) / sum(extended read blocks len)).
    """
    assert len(blocks) >= 1

    # Not optimized, naive algorithm.
    # We can do better by working with intervals + values, but this will take way too much effort for now.
    saveto = np.zeros(contiglen, dtype=np.float32)
    for b in blocks:
        assert len(b.records) >= 2, b.records
        assert (
                len(b.start) == len(b.end) == b.records[-1]
        ), f"{len(b.start)}, {len(b.end)}, {b.records[-5:]}"
        _pileup(b.start, b.end, b.records, extension, saveto)

    maxbreaks = sum(x.start.size for x in blocks)
    interend, values = _simplify(saveto, sensitivity, maxbreakpoints=maxbreaks)

    # Remove directly to free memory asap
    del saveto

    return Pileup(contig, interend, values)


class PileupUnitTests(unittest.TestCase):
    def test_pileup_empty(self):
        blocks = AlignedBlocks(
            "+",
            np.asarray([], dtype=np.int32),
            np.asarray([], dtype=np.int32),
            np.asarray([], dtype=np.int32),
        )
        self.assertRaises(AssertionError, calculate, [blocks], 0)
        self.assertRaises(AssertionError, calculate, [blocks], 10)

    def _test(self, blocks, contiglen, extension, results):
        workload = []
        for b in blocks:
            workload.append(AlignedBlocks.from_tuples(".", b))

        before = deepcopy(workload)

        res = calculate(".", contiglen, workload, extension)
        self.assertTrue(res.id == ".")
        interend, values = zip(*results)
        np.testing.assert_array_equal(res.interend, interend)
        np.testing.assert_almost_equal(res.values, values, decimal=6)

        for b, a in zip(before, workload):
            np.testing.assert_array_equal(b.start, a.start)
            np.testing.assert_array_equal(b.end, a.end)
            np.testing.assert_array_equal(b.records, a.records)

    def test_pileup(self):
        reads = [
            [(1, 4), (5, 7), (9, 11)],
            [(15, 18)],
            [(1, 2)],
            [(6, 7), (8, 10)],
            [(12, 13), (14, 15), (16, 17)],
        ]
        results = [
            (1, 0),
            (2, 2),
            (4, 1),
            (5, 0),
            (6, 1),
            (7, 2),
            (8, 0),
            (9, 1),
            (10, 2),
            (11, 1),
            (12, 0),
            (13, 1),
            (14, 0),
            (16, 1),
            (17, 2),
            (18, 1),
        ]
        self._test([reads], 18, 0, results)

    # def test_pileup_baseline(self):
    #     reads = [[(0, 1)], [(1, 2)], [(2, 3)], [(3, 4)], [(4, 5)], [(0, 5)]]
    #     results = [(5, 2)]
    #     for baseline in 0, 1, 2:
    #         self._test(reads, 5, 0, results)
    #
    #     for baseline, results in (0, [(5, 2), (10, 0)]), (1, [(5, 2), (10, 1)]), (2, [(10, 2)]):
    #         self._test(reads, 10, 0, results)
    #
    #     reads = [[(1, 2)], [(2, 3)], [(3, 4)], [(4, 5)], [(4, 5)]]
    #     for baseline, results in (0, [(1, 0), (4, 1), (5, 2)]), (1, [(4, 1), (5, 2)]), (2, [(5, 2)]):
    #         self._test(reads, 5, 0, results)
    #
    #     for baseline, results in (0, [(1, 0), (4, 1), (5, 2), (7, 0)]), (1, [(4, 1), (5, 2), (7, 1)]), (2, [(7, 2)]):
    #         self._test(reads, 7, 0, results)
    #
    #     reads = [[(1, 2)], [(2, 3)], [(3, 4)], [(4, 5)], [(4, 5)]]
    #     results = [(1, 0), (4, 1), (5, 2)]
    #     self._test(reads, 5, 0, results)

    def test_pileup_extension(self):
        reads = [[(2, 4), (6, 8)]]

        # no extension
        result = [(2, 0), (4, 1), (6, 0), (8, 1), (15, 0)]
        self._test([reads], 15, 0, result)
        # 1bp
        val = 4 / 6
        result = [(1, 0), (4, val), (6, 0), (9, val), (15, 0)]
        self._test([reads], 15, 1, result)
        # full coverage
        val = 4 / 13
        result = [(4, val), (6, 0), (15, val)]
        self._test([reads], 15, 100, result)

        # complex - 1
        reads = [
            # sample 1
            [[(1, 2), (3, 4), (5, 6)]],
            # sample 2
            [[(2, 7), (14, 15)], [(8, 11)]],
        ]
        result = [
            (1, 3 / 5),
            (2, 3 / 5 + 6 / 7),
            (3, 6 / 7),
            (4, 3 / 5 + 6 / 7),
            (5, 6 / 7),
            (7, 3 / 5 + 6 / 7),
            (12, 3 / 5),
            (14, 0),
            (15, 6 / 7),
        ]
        self._test(reads, 15, 1, result)
        # complex - 2
        reads.append([[(0, 1)], [(10, 12)]])
        result = [
            (2, 3 / 6 + 6 / 8 + 1 / 3),
            (3, 6 / 8 + 1 / 3),
            (4, 3 / 6 + 6 / 8),
            (5, 6 / 8),
            (6, 3 / 6 + 6 / 8),
            (7, 3 / 6 + 6 / 8 + 3 / 7),
            (8, 3 / 6 + 3 / 7),
            (13, 3 / 7 + 2 / 6),
            (14, 2 / 6),
            (15, 6 / 8),
        ]
        self._test(reads, 15, 2, result)
