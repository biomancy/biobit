import bisect
from dataclasses import dataclass

import numpy as np
import numpy.typing as npt

from ..pileup.pileup import Pileup


@dataclass()
class Track:
    # For i-th interval, bounds[i] = start, bounds[i+1] = end
    # i-th interval = [bounds[i], bounds[i + 1])
    bounds: npt.NDArray[np.int32]
    values: npt.NDArray[np.float32]

    def __post_init__(self):
        assert self.bounds.size >= 2 and self.bounds.size - 1 == self.values.size
        assert np.all(self.bounds[1:] > self.bounds[:-1])

    @staticmethod
    def from_pileup(interend: npt.NDArray[np.int32], values: npt.NDArray[np.float32]):
        assert interend[0] != 0
        return Track(np.insert(interend, 0, 0), values)

    def at(self, pos: np.int32) -> np.float32:
        assert 0 <= pos < self.bounds[-1]
        ind = bisect.bisect_left(self.bounds, pos)
        assert self.bounds[ind] <= pos < self.bounds[ind + 1]
        return self.values[ind]

    def maximum(self, start: int, end: int) -> tuple[np.int32, np.float32]:
        left = bisect.bisect_right(self.bounds, start)
        if left > 0:
            left -= 1
        right = bisect.bisect_left(self.bounds, end)
        assert self.bounds[left] <= start <= end <= self.bounds[right]
        assert right > left >= 0
        if right - 1 == left:
            argmax = left
        else:
            argmax = int(self.values[left:right].argmax()) + left
        # Middle of the region
        return (self.bounds[argmax] + self.bounds[argmax + 1]) // 2, self.values[argmax]


@dataclass(frozen=True)
class Result:
    contig: str
    contiglen: int
    trstrand: str

    track: Track

    @staticmethod
    def from_pileup(pileup: Pileup, contiglen: int, trstrand: str) -> "Result":
        track = Track.from_pileup(pileup.interend, pileup.values)
        return Result(pileup.id, contiglen, trstrand, track)


@dataclass(frozen=True)
class Peak:
    contig: str
    start: int
    end: int
    strand: str
    pvalue: float
    qvalue: float
    fe: float
    summit: list[int]
