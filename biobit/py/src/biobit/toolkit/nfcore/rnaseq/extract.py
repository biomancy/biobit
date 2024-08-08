from pathlib import Path
from typing import Callable

from biobit import io
from biobit.core.ngs import Layout
from biobit.toolkit import seqproj


def default(path: str, layout: Layout) -> io.bam.Reader:
    if isinstance(layout, Layout.Single):
        return io.bam.Reader(path, inflags=0, exflags=2564, minmapq=0)
    elif isinstance(layout, Layout.Paired):
        return io.bam.Reader(path, inflags=3, exflags=2564, minmapq=0)
    else:
        raise ValueError(f"Unsupported layout: {layout}")


BamReader = Callable[[str, Layout], io.bam.Reader]


def bam(experiment: seqproj.Experiment, /, factory: BamReader = default) -> tuple[io.bam.Reader, Layout]:
    if "__nfcore_rnaseq_bam__" not in experiment.attributes:
        raise ValueError(f"Attribute '__nfcore_rnaseq_bam__' not found for the experimetn: {experiment}")
    layout = experiment.ngs()
    return factory(experiment.attributes["__nfcore_rnaseq_bam__"], layout), layout


def bigwig(experiment: seqproj.Experiment, /) -> tuple[Path, Path]:
    for attr in "__nfcore_rnaseq_bigwig_fwd__", "__nfcore_rnaseq_bigwig_rev__":
        if attr not in experiment.attributes:
            raise ValueError(f"Attribute '{attr}' not found for the experimetn: {experiment}")

    fwd = Path(experiment.attributes["__nfcore_rnaseq_bigwig_fwd__"])
    rev = Path(experiment.attributes["__nfcore_rnaseq_bigwig_rev__"])
    return fwd, rev


def bams(project: seqproj.Project, /, factory: BamReader = default) -> dict[str, tuple[io.bam.Reader, Layout]]:
    return {exp.ind: bam(exp, factory=factory) for exp in project.experiments}
