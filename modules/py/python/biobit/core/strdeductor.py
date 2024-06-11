from typing import Callable, Literal

from pysam import AlignedSegment

from biobit.analysis import seqproj
from .strand import Strand

StrandDeductor = Callable[[AlignedSegment], Strand]


def get(protocol: Literal["f/s", "s/f", "f", "s"]) -> StrandDeductor:
    match protocol:
        case "f/s":
            return _fs
        case "s/f":
            return _sf
        case "f":
            return _f
        case "s":
            return _s
        case _:
            raise ValueError(f"Unknown stranding protocol: {protocol}")


def from_bioproj(experiment: seqproj.Experiment) -> StrandDeductor:
    runs = list(experiment.runs)
    layout = runs[0].layout
    for r in runs[1:]:
        if r.layout != layout:
            raise ValueError(
                f"All runs must have the same layout. Found different layouts: {layout} and {r.layout} in {experiment}"
            )

    match (layout, experiment.library.stranding):
        case (seqproj.SeqLayout.Single, seqproj.Stranding.Forward):
            return _s
        case (seqproj.SeqLayout.Single, seqproj.Stranding.Reverse):
            return _f
        case (seqproj.SeqLayout.Paired, seqproj.Stranding.Forward):
            return _sf
        case (seqproj.SeqLayout.Paired, seqproj.Stranding.Reverse):
            return _fs
        case _:
            raise ValueError(
                f"Unsupported layout/stranding combination: {layout}/{experiment.library.stranding}\n"
                f"bioproj.Experiment: {experiment}"
            )


def _f(read: AlignedSegment) -> Strand:
    # Flip the mapped strand
    match read.is_reverse:
        case True:
            return Strand.fwd
        case False:
            return Strand.rev
        case _:
            assert False, "Unreachable"


def _s(read: AlignedSegment) -> Strand:
    # Keep the mapped strand
    match read.is_reverse:
        case True:
            return Strand.rev
        case False:
            return Strand.fwd
        case _:
            assert False, "Unreachable"


def _fs(read: AlignedSegment) -> Strand:
    match (read.is_read1, read.is_reverse):
        # First mate -> flip its strand
        case (True, True):
            return Strand.fwd
        case (True, False):
            return Strand.rev
        # Second mate -> keep its strand
        case (False, True):
            return Strand.rev
        case (False, False):
            return Strand.fwd
        case _:
            assert False, "Unreachable"


def _sf(read: AlignedSegment) -> Strand:
    match (read.is_read1, read.is_reverse):
        # First mate -> keep its strand
        case (True, True):
            return Strand.rev
        case (True, False):
            return Strand.fwd
        # Second mate -> flip its strand
        case (False, True):
            return Strand.fwd
        case (False, False):
            return Strand.rev
        case _:
            assert False, "Unreachable"
