import numpy as np
import numpy.typing as npt
from MACS3.Signal.Prob import poisson_cdf

from .result import Result, Track
from ..pipeline import pipeline

FILTERED_PQVALUE = np.float32(-1)


# @numba.jit(cache=True, nopython=True, nogil=True)
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

    # Cast all treatment pileups to int
    trtvalues = trtvalues.astype(np.int32)

    # P-values lookup table
    ptables = {}
    # Stat for q-values calculation
    pvalue_counts: dict[np.float32, int] = {}

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

        if trtval == 0:
            # Ignore low covered regions
            values[i] = FILTERED_PQVALUE
        else:
            # Calculate p-values only for well-covered regions
            assert cntval >= 0
            key = (trtval, cntval)
            if key not in ptables:
                ptables[key] = np.float32(
                    -1 * poisson_cdf(trtval - 1, cntval, False, True)
                )

            pv = ptables[key]
            values[i] = pv
            pvalue_counts[pv] = pvalue_counts.get(pv, 0) + (bounds[i + 1] - bounds[i])

        i += 1
        if bounds[i] == chromsize:
            break
    bounds, values = bounds[: i + 1], values[:i]
    return bounds, values, pvalue_counts


def calculate(workload: pipeline.Results) -> tuple[Result, dict[float, int]]:
    bounds, values, pvcounts = _job(
        workload.cntpileup.interend,
        workload.cntpileup.values,
        workload.trtpileup.interend,
        workload.trtpileup.values,
    )

    track = Result(
        workload.contig, workload.contiglen, workload.trstrand, Track(bounds, values)
    )
    return track, pvcounts
