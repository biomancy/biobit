import copy
from itertools import chain

from joblib import Parallel, delayed

from . import core
from .core import pipeline
from .core.config import Config


def run(config: Config):
    with Parallel(
            n_jobs=config.process.threads, backend=config.process.backend
    ) as pool:
        pileups = pipeline.run(config, pool)

        # Convert to tracks and save pileups
        if config.saveto.pileup:
            for key, title in (
                    (lambda x: x.trtpileup, f"{config.saveto.title}.trt"),
                    (lambda x: x.cntpileup, f"{config.saveto.title}.cnt"),
            ):
                tracks = [
                    core.functors.Result.from_pileup(key(x), x.contiglen, x.trstrand)
                    for x in pileups
                ]
                core.io.tobigwig(tracks, config.saveto.pileup, title)
                del tracks

        # Calculate fold enrichment
        # fe = [core.functors.foldenrichment.calculate(w) for w in pileups]
        fe = pool(delayed(core.functors.foldenrichment.calculate)(w) for w in pileups)
        if config.saveto.enrichment:
            core.io.tobigwig(fe, config.saveto.enrichment, config.saveto.title)

        if (
                config.saveto.pvpeaks is None
                and config.saveto.fdrpeaks is None
                and config.saveto.pvtrack is None
        ):
            return

        # Calculate p-values
        # pvalues = [core.functors.pvalues.calculate(w) for w in pileups]
        _pvalues: list[tuple[core.functors.pvalues.Result, dict[float, int]]] = pool(
            delayed(core.functors.pvalues.calculate)(w) for w in pileups
        )
        pvalues: list[core.functors.pvalues.Result] = [x[0] for x in _pvalues]
        pcounts: list[dict[float, int]] = [x[1] for x in _pvalues]

        if config.saveto.pvtrack is not None:
            core.io.tobigwig(pvalues, config.saveto.pvtrack, config.saveto.title)

        # Calculate q-values
        pqtable = core.functors.qvalues.make_pqtable(pcounts)
        # qvalues = [core.functors.qvalues.apply_pqtable(w, pqtable) for w in pvalues]
        qvalues = pool(
            delayed(core.functors.qvalues.apply_pqtable)(w, pqtable) for w in pvalues
        )
        # core.io.tobigwig(pvalues, config.saveto.enrichment, f"{config.saveto.title}.qvalue")

        # Call peaks using various cutoffs
        workload = []
        if config.saveto.pvpeaks and config.callp.pvcutoff is not None:
            callp = copy.deepcopy(config.callp)
            object.__setattr__(callp, "qvcutoff", None)
            workload.append((callp, config.saveto.pvpeaks))
        if config.saveto.fdrpeaks and config.callp.qvcutoff is not None:
            callp = copy.deepcopy(config.callp)
            object.__setattr__(callp, "pvcutoff", None)
            workload.append((callp, config.saveto.fdrpeaks))

        for callp, saveto in workload:
            peak_calling_workload = core.functors.callpeaks.PeakCalingWorkload.build(
                pvalues, qvalues, fe, callp
            )
            # peaks = [core.functors.callpeaks.calculate(w) for w in workload]
            peaks = pool(
                delayed(core.functors.callpeaks.calculate)(w) for w in peak_calling_workload
            )
            peaks = list(chain(*peaks))

            core.io.tobed(peaks, saveto.joinpath(f"{config.saveto.title}.narrowPeak"))
