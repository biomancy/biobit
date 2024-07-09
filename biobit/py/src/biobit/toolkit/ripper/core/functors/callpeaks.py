from collections import defaultdict
from dataclasses import dataclass

import numpy as np

from .result import Peak, Result, Track
from .. import config


@dataclass(frozen=True)
class Context:
    qvalues: Track
    pvalues: Track
    foldenrichment: Track

    def __post_init__(self):
        assert np.all(self.qvalues.bounds == self.foldenrichment.bounds) and np.all(
            self.qvalues.bounds == self.pvalues.bounds
        )


@dataclass(frozen=True)
class PeakCalingWorkload:
    contig: str
    contiglen: np.int32
    trstrand: str

    params: config.PeakCalling
    ctx: Context

    @staticmethod
    def build(
            pv: list[Result], qv: list[Result], fe: list[Result], params: config.PeakCalling
    ) -> list["PeakCalingWorkload"]:
        # Group by chromosomes
        assert (
                set((x.contig, x.trstrand) for x in pv)
                == set((x.contig, x.trstrand) for x in qv)
                == set((x.contig, x.trstrand) for x in fe)
        )
        assert len(pv) == len(qv) == len(fe)
        grouped: dict[tuple[str, str], dict[str, Result]] = defaultdict(dict)
        for key, elements in {"pv": pv, "qv": qv, "fe": fe}.items():
            for x in elements:
                grouped[(x.contig, x.trstrand)][key] = x

        # Build workloads
        workloads = []
        for (contig, trstrand), data in grouped.items():
            _pv, _qv, _fe = data.pop("pv"), data.pop("qv"), data.pop("fe")
            ctx = Context(_qv.track, _pv.track, _fe.track)
            workloads.append(
                PeakCalingWorkload(contig, np.int32(_pv.contiglen), trstrand, params, ctx)
            )
        return workloads


@dataclass(frozen=True)
class PeakPiece:
    start: int
    end: int
    pvalue: float
    qvalue: float
    fe: float


def _stitch(
        pieces: list[PeakPiece],
        contig: str,
        strand: str,
        params: config.PeakCalling,
        ctx: Context,
) -> Peak | None:
    if not pieces:
        return None

    # Pieces must be ordered
    assert all(x.end <= y.start for x, y in zip(pieces[:-1], pieces[1:]))

    # Skip small peaks
    if pieces[-1].end - pieces[0].start < params.minsize:
        return None

    # Summit = position with a max fold enrichment
    maxfe = max(pieces, key=lambda x: x.fe).fe
    summits = [(x.start + x.end) // 2 for x in pieces if x.fe == maxfe]

    # Get the piece with min p/q-value (=max log10 p/q-value)
    x = max(pieces, key=lambda x: x.qvalue)
    minqv, minpv = x.qvalue, x.pvalue
    return Peak(
        contig, pieces[0].start, pieces[-1].end, strand, minpv, minqv, maxfe, summits
    )


def calculate(w: PeakCalingWorkload) -> list[Peak]:
    mask = np.ones_like(w.ctx.pvalues.values, dtype=bool)
    if w.params.qvcutoff:
        qvcutoff = -np.log10(w.params.qvcutoff)
        mask = mask & (w.ctx.qvalues.values >= qvcutoff)
    if w.params.fecutoff:
        mask = mask & (w.ctx.foldenrichment.values >= w.params.fecutoff)
    if w.params.pvcutoff:
        pvcutoff = -np.log10(w.params.pvcutoff)
        mask = mask & (w.ctx.pvalues.values >= pvcutoff)

    peakind = np.nonzero(mask)[0]
    if peakind.size == 0:
        return []

    qv, pv, fe = (
        w.ctx.qvalues.values[peakind],
        w.ctx.pvalues.values[peakind],
        w.ctx.foldenrichment.values[peakind],
    )
    starts, ends = w.ctx.qvalues.bounds[peakind], w.ctx.qvalues.bounds[peakind + 1]

    result = []
    buffer = [PeakPiece(starts[0], ends[0], pv[0], qv[0], fe[0])]
    for ind in range(1, peakind.size):
        piece = PeakPiece(starts[ind], ends[ind], pv[ind], qv[ind], fe[ind])
        assert piece.start - buffer[-1].end >= 0

        if piece.start - buffer[-1].end <= w.params.maxgap:
            buffer.append(piece)
        else:
            peak = _stitch(buffer, w.contig, w.trstrand, w.params, w.ctx)
            result.append(peak)
            # Reset buffer
            buffer = [piece]

    if buffer:
        result.append(_stitch(buffer, w.contig, w.trstrand, w.params, w.ctx))
    filtered = [x for x in result if x is not None]
    return filtered
