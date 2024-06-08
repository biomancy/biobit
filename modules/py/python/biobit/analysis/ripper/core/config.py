from pathlib import Path
from typing import Literal

import numpy as np
from attrs import define, field


@define(slots=True, frozen=True)
class Scaling:
    treatment: np.float32
    control: np.float32


@define(slots=True, frozen=True)
class Saveto:
    title: str
    pileup: Path | None
    enrichment: Path | None
    pvtrack: Path | None
    pvpeaks: Path | None
    fdrpeaks: Path | None


@define(slots=True, frozen=True)
class Pileup:
    # Stranding (only f/s & s/f are supported)
    stranding: Literal["f/s", "s/f"]
    # Scaling options
    scaling: Scaling
    # Extension size for each contig (both transcriptomic & genomic coordinates)
    extsize: dict[str, tuple[int, ...]]
    # Number of reads to parallelize the pileup computation
    threads: int
    backend: str
    # Bam flags
    inflags: int
    exflags: int
    minmapq: int


@define(slots=True, frozen=True)
class PeakCalling:
    qvcutoff: float | None = field(default=0.01)
    pvcutoff: float | None = field(default=None)
    fecutoff: float | None = field(default=2)
    minsize: int = field(default=50)
    maxgap: int = field(default=25)
    mintrtfrag: int = field(default=10)

    def __attrs_post_init__(self):
        if self.qvcutoff is None and self.pvcutoff is None and self.fecutoff is None:
            raise ValueError("At least one of [qvcutoff, pvcutoff, fecutoff] must be set")


@define(slots=True, frozen=True)
class Config:
    # Input bam files
    treatment: tuple[Path, ...]
    control: tuple[Path, ...]
    # Target contigs
    contigs: tuple[str, ...] | None
    # Genome size
    geffsize: int

    process: Pileup
    callp: PeakCalling
    saveto: Saveto

    def __attrs_post_init__(self):
        for x in self.treatment + self.control:
            if not x.is_file():
                raise FileNotFoundError(f"File {x} is missing")
