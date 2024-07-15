from collections.abc import Iterable
from pathlib import Path
from typing import TypeVar, Callable

from biobit import io
from biobit.toolkit import nfcore
from .countit import CountIt
from ...core.ngs import Layout

_K = TypeVar('_K')


def default_se_bam_reader(path: Path) -> io.bam.Reader:
    return io.bam.Reader(path, inflags=0, exflags=2564, minmapq=0)


def default_pe_bam_reader(path: Path) -> io.bam.Reader:
    return io.bam.Reader(path, inflags=3, exflags=2564, minmapq=0)


def default_key(prj: nfcore.rnaseq.Project, experiment: nfcore.rnaseq.Experiment) -> tuple[str, str]:
    return prj.ind, experiment.ind


def from_nfcore_rnaseq(
        countit: CountIt, projects: Iterable[nfcore.rnaseq.Project], *,
        se_bam: Callable[[Path], io.bam.Reader] = default_se_bam_reader,
        pe_bam: Callable[[Path], io.bam.Reader] = default_pe_bam_reader,
        key: Callable[[nfcore.rnaseq.Project, nfcore.rnaseq.Experiment], _K] = default_key,  # type: ignore
) -> CountIt:
    for prj in projects:
        for experiment in prj.experiments:
            tag = key(prj, experiment)
            ngs = experiment.ngs()

            if isinstance(ngs, Layout.Single):
                bam = se_bam(experiment.bam)
            elif isinstance(ngs, Layout.Paired):
                bam = pe_bam(experiment.bam)
            else:
                raise ValueError(f"Unsupported layout: {ngs}")

            countit.add_source(tag, bam, ngs)
    return countit
