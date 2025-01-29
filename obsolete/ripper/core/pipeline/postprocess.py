import logging
from dataclasses import dataclass
from typing import Any, Optional, Tuple

import numpy as np

from biobit.core import Stranded
from .pileup import Results
from .. import pileup


@dataclass(frozen=True)
class Workload:
    pileup: Results

    # Baseline values
    gmbaseline: np.float32
    minfragments: np.float32
    # Scaling coefficient
    scale: Optional[np.float32]


@dataclass(frozen=True)
class Result:
    contig: str
    contiglen: np.int32

    pileup: Stranded[pileup.Pileup]
    tags: Any


def calcscaling(trtfragments: int, cntfragments: int) -> Tuple[str, np.float32]:
    print(f"Treatment fragments: {trtfragments}, Control fragments: {cntfragments}")

    if trtfragments > cntfragments:
        scale = cntfragments / trtfragments
        scalelib = "treatment"
        logging.info(
            f"Treatment ({trtfragments}) is large than control ({cntfragments}), scaling multiplier: {scale}"
        )
    else:
        scale = trtfragments / cntfragments
        scalelib = "control"
        logging.info(
            f"Control ({cntfragments}) is large than treatment ({trtfragments}), scaling multiplier: {scale}"
        )
    return scalelib, np.float32(scale)


def _normalize(
        p: Stranded[pileup.Pileup], scale: Optional[np.float32]
) -> Stranded[pileup.Pileup]:
    if scale:
        p.fwd.values *= scale
        p.rev.values *= scale
    return p


def run(workload: Workload) -> Result:
    # Apply baseline value
    result: Stranded[pileup.Pileup] = Stranded(
        fwd=pileup.merge.by_max([workload.pileup.genomic.fwd], baseline=workload.gmbaseline),
        rev=pileup.merge.by_max([workload.pileup.genomic.rev], baseline=workload.gmbaseline),
    )

    # Discard low-covered regions
    if workload.minfragments > 0:
        for x in result.fwd, result.rev:
            mask = x.values < workload.minfragments
            x.values[mask] = np.float32(0)

    # Normalize libraries
    result = _normalize(result, workload.scale)
    return Result(
        contig=workload.pileup.contig,
        contiglen=workload.pileup.contiglen,
        pileup=result,
        tags=workload.pileup.tags,
    )
