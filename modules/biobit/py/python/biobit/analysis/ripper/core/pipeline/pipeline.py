import logging
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import List, Iterable, Any

import numpy as np
from joblib import Parallel, delayed
from pysam import AlignmentFile

from . import pileup, postprocess
from .. import config
from ..pileup import Pileup

TREATMENT = "treatment"
CONTROL = "control"


@dataclass(frozen=True)
class Results:
    contig: str
    contiglen: int
    trstrand: str

    trtpileup: Pileup
    cntpileup: Pileup

    def __post_init__(self):
        assert self.trtpileup.id == self.cntpileup.id == self.contig


def fetch_contigs(inbam: Iterable[Path]) -> tuple[str, ...]:
    contigs = set()
    for b in inbam:
        for contig in AlignmentFile(b.as_posix(), "rb").references:
            contigs.add(contig)
    return tuple(contigs)


def run(cnf: config.Config, pool: Parallel) -> List[Results]:
    # Parse BAM contigs
    contigs = (
        cnf.contigs
        if cnf.contigs
        else fetch_contigs(cnf.treatment + cnf.control)
    )

    # Build & run pileup workloads
    workloads = []
    prconfigs = {
        CONTROL: cnf.process,
        # Disable extension for treatment
        TREATMENT: config.Pileup(
            cnf.process.stranding,
            cnf.process.scaling,
            defaultdict(lambda *args: (0,)),
            cnf.process.threads,
            cnf.process.backend,
            cnf.process.inflags,
            cnf.process.exflags,
            cnf.process.minmapq,
        ),
    }
    for contig in contigs:
        for tag, files in {
            TREATMENT: cnf.treatment,
            CONTROL: cnf.control,
        }.items():
            workloads.append(
                pileup.Workload(
                    contig=contig, bamfiles=list(files), params=prconfigs[tag], tags=tag
                )
            )
    results: List[pileup.Results] = pool(delayed(pileup.run)(w) for w in workloads)

    # Calculate baseline values
    trtfragments = sum(x.fragments for x in results if x.tags == TREATMENT)
    cntfragments = sum(x.fragments for x in results if x.tags == CONTROL)

    gmbaseline = cntfragments / cnf.geffsize
    print(f"Treatment fragments: {trtfragments}, Control fragments: {cntfragments}")
    print(
        f"Treatment scaling: {cnf.process.scaling.treatment}, "
        f"Control scaling: {cnf.process.scaling.control}"
    )
    logging.info(f"Genome baseline: {gmbaseline}")

    # Build & run postprocess workloads
    postprocess_workloads = []
    baselines = {TREATMENT: (0.0, cnf.callp.mintrtfrag), CONTROL: (gmbaseline, 0.0)}
    for r in results:
        gmbaseline, minfragments = baselines[r.tags]

        if r.tags == TREATMENT:
            scale = cnf.process.scaling.treatment
        else:
            assert r.tags == CONTROL
            scale = cnf.process.scaling.control

        postprocess_workloads.append(
            postprocess.Workload(
                pileup=r,
                gmbaseline=np.float32(gmbaseline),
                scale=scale,
                minfragments=np.float32(minfragments),
            )
        )
    postprocess_results: List[postprocess.Result] = pool(
        delayed(postprocess.run)(w) for w in postprocess_workloads
    )
    # results: List[postprocess.Result] = [postprocess.run(w) for w in workloads]

    # Regroup results
    regrouped: dict[tuple[str, np.int32, str], dict[Any, Pileup]] = defaultdict(dict)
    for pr in postprocess_results:
        # Forward
        key = (pr.contig, pr.contiglen, "+")
        assert key not in regrouped or pr.tags not in regrouped[key]
        regrouped[key][pr.tags] = pr.pileup.fwd
        # Reverse
        key = (pr.contig, pr.contiglen, "-")
        assert key not in regrouped or pr.tags not in regrouped[key]
        regrouped[key][pr.tags] = pr.pileup.rev

    # Final result
    outcome = []
    for (contig, contiglen, trstrand), pileups in regrouped.items():
        if CONTROL not in pileups or TREATMENT not in pileups:
            nonein = "control" if CONTROL not in pileups else "treatment"
            logging.warning(
                f"No {nonein} fragments for contig {contig}, strand {trstrand}"
            )
            continue
        outcome.append(
            Results(
                contig=contig,
                contiglen=int(contiglen),
                trstrand=trstrand,
                trtpileup=pileups[TREATMENT],
                cntpileup=pileups[CONTROL],
            )
        )
    return outcome
