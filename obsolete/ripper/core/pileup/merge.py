import unittest

import numba
import numpy as np
import numpy.typing as npt
from numba import float32, int32

from .pileup import Pileup

SENSITIVITY = float32(1e-5)


@numba.jit(cache=True, nopython=True, nogil=True)
def _simplify(interend, values, sensitivity: float32 = SENSITIVITY):
    assert interend.size == values.size and interend.size > 0
    curval, curend, writepos = values[0], interend[0], 0
    for ind in range(1, interend.size):
        # Skip elements if diff is small
        if abs(values[ind] - curval) < sensitivity:
            curend = interend[ind]
            continue
        interend[writepos] = curend
        values[writepos] = curval
        writepos += 1

        curval, curend = values[ind], interend[ind]

    interend[writepos] = curend
    values[writepos] = curval
    finallen = writepos + 1
    assert finallen <= interend.size

    return interend[:finallen], values[:finallen]


@numba.jit(cache=True, nopython=True, nogil=True)
def _by_max(
        ends_of_intervals: list[npt.NDArray[np.int32]],
        values: list[npt.NDArray[np.float32]],
        baseline: float32,
):
    for p, v in zip(ends_of_intervals, values):
        assert p.size == v.size

    # 1. Make buffers for current intervals and results
    tracks = len(ends_of_intervals)
    track_nextind = np.zeros(tracks, dtype=np.int32)
    track_curent = np.empty(tracks, dtype=np.int32)
    track_curval = np.empty(tracks, dtype=np.float32)

    for track in range(tracks):
        track_curent[track] = ends_of_intervals[track][0]
        track_curval[track] = values[track][0]

    maxlength = int32(0)
    for p in ends_of_intervals:
        maxlength += p.size
    res_ends = np.empty(maxlength, dtype=np.int32)
    res_values = np.empty(maxlength, dtype=np.float32)

    curval = max(baseline, np.max(track_curval))
    nextval = curval

    curend = min(track_curent)
    real_length = 0

    # ahead declaration of buffer to avoid memory allocation on each iteration
    to_drop = np.empty(tracks, dtype=np.int32)
    while tracks > 0:
        # 1. Next value is a max among present intervals and baseline value
        nextval = max(baseline, np.max(track_curval))

        # 2. End of the interval is a min among active intervals ends
        nextend = min(track_curent)

        # 3. Save interval if new value is encountered
        if nextval != curval:
            res_values[real_length] = curval
            res_ends[real_length] = curend
            real_length += 1
            curval = nextval
        curend = nextend

        # 4. Push intervals if needed and drop finished intervals
        to_drop_total = 0
        for track in range(tracks):
            if track_curent[track] == nextend:
                nextind = track_nextind[track] + 1
                # finished interval
                if nextind == ends_of_intervals[track].size:
                    to_drop[to_drop_total] = track
                    to_drop_total += 1
                    continue
                track_curent[track] = ends_of_intervals[track][nextind]
                track_curval[track] = values[track][nextind]
                track_nextind[track] = nextind

                # update max value for the current interval
                # if values[track][nextind - 1] == nextval:
                #     nextval = max(baseline, np.max(track_curval))
                # else:
                #     nextval = max(nextval, track_curval[track])

        if to_drop_total != 0:
            tracks -= to_drop_total
            indices = to_drop[:to_drop_total]
            track_curent = np.delete(track_curent, indices)
            track_curval = np.delete(track_curval, indices)
            track_nextind = np.delete(track_nextind, indices)

            for track in to_drop[:to_drop_total][::-1]:
                assert (
                        track < len(ends_of_intervals)
                        and len(ends_of_intervals) == len(values)
                        and len(ends_of_intervals) > 0
                )
                ends_of_intervals.pop(track)
                values.pop(track)

    res_values[real_length] = curval
    res_ends[real_length] = curend
    real_length += 1

    assert res_ends.size >= real_length
    return res_ends[:real_length], res_values[:real_length]


def by_max(pileups: list[Pileup], baseline: float32 | None = None) -> Pileup:
    assert len(pileups) > 0 and all(
        x.id == pileups[0].id and x.interend[-1] == pileups[0].interend[-1]
        for x in pileups
    )

    if len(pileups) == 1:
        pileup = pileups[0]
        if baseline:
            pileup.owned()
            np.maximum(pileup.values, baseline, out=pileup.values)
            pileup.interend, pileup.values = _simplify(pileup.interend, pileup.values)
        return pileup.owned()

    # All arrays must be typed and have the same "constness"
    interends, values = numba.typed.List(), numba.typed.List()
    for x in pileups:
        x.interend.setflags(write=False)
        interends.append(x.interend)
        x.values.setflags(write=False)
        values.append(x.values)

    baseline = baseline if baseline else np.float32(0)
    ends, values = _by_max(interends, values, baseline)
    return Pileup(pileups[0].id, ends, values).owned()


class MergeByMaxUnitTests(unittest.TestCase):
    def _test(self, pileups, baseline, expected):
        workload = [Pileup.from_tuples("", p) for p in pileups]
        result = by_max(workload, baseline)

        expected = Pileup.from_tuples("", expected)
        np.testing.assert_array_equal(result.interend, expected.interend)
        np.testing.assert_array_equal(result.values, expected.values)

    def test_by_max_empty(self):
        self.assertRaises(AssertionError, by_max, [], 123)

    def test_by_max_inconsistent(self):
        pileups = [
            Pileup(
                "1", np.asarray([12], dtype=np.int32), np.asarray(9, dtype=np.float32)
            ),
            Pileup(
                "1", np.asarray([1], dtype=np.int32), np.asarray(1, dtype=np.float32)
            ),
        ]
        self.assertRaises(AssertionError, by_max, pileups, 0)

        pileups = [
            Pileup(
                "1", np.asarray([12], dtype=np.int32), np.asarray(9, dtype=np.float32)
            ),
            Pileup(
                "2", np.asarray([12], dtype=np.int32), np.asarray(1, dtype=np.float32)
            ),
        ]
        self.assertRaises(AssertionError, by_max, pileups, 0)

    def test_by_max_single(self):
        # without baseline
        pileup = [(10, 1), (25, 13)]
        self._test([pileup], 0, pileup)
        # with baseline
        expected = [(25, 156)]
        self._test([pileup], 156, expected)

        pileup = [(5, 0), (9, 1), (10, 0), (14, 3), (25, 100), (26, 0), (27, 1)]
        # baseline = 0
        self._test([pileup], 0, pileup)
        # baseline = 1
        expected = [(10, 1), (14, 3), (25, 100), (27, 1)]
        self._test([pileup], 1, expected)
        # baseline = 3
        expected = [(14, 3), (25, 100), (27, 3)]
        self._test([pileup], 3, expected)
        # baseline = 1000
        expected = [(27, 1000)]
        self._test([pileup], 1000, expected)

    def test_by_max_overlaping_ends(self):
        pileups = [
            [(5, 1), (6, 2), (7, 19), (10, 4), (25, 0)],
            [(6, 10), (7, 10), (10, 5), (25, 9)],
        ]
        expected = [(6, 10), (7, 19), (10, 5), (25, 9)]
        self._test(pileups, 0, expected)
        # more + baseline
        pileups = pileups + pileups + pileups
        expected = [(6, 10), (7, 19), (10, 6), (25, 9)]
        self._test(pileups, 6, expected)

    def test_by_max_complex(self):
        # case 1
        pileups = [
            [(1, 0), (4, 3), (6, 2), (8, 9), (10, 0)],
            [(2, 1), (7, 10), (10, 1)],
            [(4, 3), (6, 1), (8, 8), (10, 1)],
        ]
        expected = [(2, 3), (7, 10), (8, 9), (10, 1)]
        self._test(pileups, 0, expected)
        expected = [(2, 3), (7, 10), (8, 9), (10, 3)]
        self._test(pileups, 3, expected)

        # case 2
        pileups = [
            [(2, 1), (4, 2), (6, 3), (8, 4), (10, 5)],
            [(2, 5), (4, 4), (6, 3), (8, 2), (10, 1)],
            [(1, 0), (3, 5), (5, 4), (7, 3), (9, 2), (10, 0)],
        ]
        expected = [(3, 5), (5, 4), (6, 3), (8, 4), (10, 5)]
        self._test(pileups, 0, expected)
        self._test(pileups, 3, expected)
        expected = [(3, 5), (8, 4), (10, 5)]
        self._test(pileups, 4, expected)
