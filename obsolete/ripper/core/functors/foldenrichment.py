import numba
import numpy as np
import numpy.typing as npt

from .result import Result, Track
from ..pipeline import pipeline


@numba.jit(cache=True, nopython=True, nogil=True)
def _job(
        cntends: npt.NDArray[np.int32],
        cntvalues: npt.NDArray[np.float32],
        trtends: npt.NDArray[np.int32],
        trtvalues: npt.NDArray[np.float32],
):
    # Chromosome must be identical
    chromsize = cntends[-1]
    assert chromsize == trtends[-1]
    # Sanity check
    assert cntends.size == cntvalues.size
    assert trtends.size == trtvalues.size

    allocate = cntends.size + trtends.size + 2
    bounds = np.empty(allocate, dtype=np.int32)
    # Start at 0
    bounds[0] = 0
    values = np.empty(allocate, dtype=np.float32)

    i, trtind, cntind = 0, 0, 0
    while True:
        trtend, cntend = trtends[trtind], cntends[cntind]
        trtval, cntval = trtvalues[trtind], cntvalues[cntind]
        if trtend == cntend:
            bounds[i + 1] = trtend
            trtind += 1
            cntind += 1
        elif trtend < cntend:
            bounds[i + 1] = trtend
            trtind += 1
        else:
            bounds[i + 1] = cntend
            cntind += 1

        assert cntval >= 0
        values[i] = trtval / cntval
        i += 1
        if bounds[i] == chromsize:
            break
    bounds, values = bounds[: i + 1], values[:i]
    return bounds, values


def calculate(workload: pipeline.Results) -> Result:
    bounds, values = _job(
        workload.cntpileup.interend,
        workload.cntpileup.values,
        workload.trtpileup.interend,
        workload.trtpileup.values,
    )
    return Result(
        workload.contig, workload.contiglen, workload.trstrand, Track(bounds, values)
    )
