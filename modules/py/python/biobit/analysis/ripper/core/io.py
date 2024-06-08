from pathlib import Path

import pyBigWig

from biobit.core import Stranded
from .functors import callpeaks
from .functors.result import Result


def _tobigwig(tracks: list[Result], bw: Path):
    print(bw)
    # There must be at least one record
    assert tracks
    # All must be on the same strand
    assert all(x.trstrand == tracks[0].trstrand for x in tracks)
    # At most one record per contig
    assert len(set(x.contig for x in tracks)) == len(tracks)

    bigwig: pyBigWig.pyBigWig = pyBigWig.open(bw.as_posix(), "w")

    # Sort by contig name
    tracks = sorted(tracks, key=lambda x: x.contig)
    # Create a header
    header = [(x.contig, x.contiglen) for x in tracks]
    bigwig.addHeader(header)
    # Add entries
    for tr in tracks:
        contigs = [tr.contig] * tr.track.values.size
        starts = tr.track.bounds[:-1]
        ends = tr.track.bounds[1:]
        values = tr.track.values
        assert values.size == starts.size == ends.size
        bigwig.addEntries(contigs, starts, ends=ends, values=values)
    bigwig.close()


def tobigwig(tracks: list[Result], folder: Path, title: str) -> Stranded[Path]:
    saveto = Stranded(
        fwd=folder.joinpath(f"{title}.fwd.bigWig"),
        rev=folder.joinpath(f"{title}.rev.bigWig"),
    )
    stranded = Stranded[list[Result]](fwd=[], rev=[])
    for t in tracks:
        if t.trstrand == "+":
            stranded.fwd.append(t)
        else:
            assert t.trstrand == "-"
            stranded.rev.append(t)
    _tobigwig(stranded.fwd, saveto.fwd)
    _tobigwig(stranded.rev, saveto.rev)
    return saveto


def tobed(peaks: list[callpeaks.Peak], saveto: Path):
    # https://genome.ucsc.edu/FAQ/FAQformat.html#format12
    # chrom - Name of the chromosome (or contig, scaffold, etc.).
    # chromStart - The starting position of the feature in the chromosome or scaffold. The first base in a chromosome is numbered 0.
    # chromEnd - The ending position of the feature in the chromosome or scaffold. The chromEnd base is not included in the display of the feature. For example, the first 100 bases of a chromosome are defined as chromStart=0, chromEnd=100, and span the bases numbered 0-99.
    # name - Name given to a region (preferably unique). Use "." if no name is assigned.
    # score - Indicates how dark the peak will be displayed in the browser (0-1000). If all scores were "'0"' when the data were submitted to the DCC, the DCC assigned scores 1-1000 based on signal value. Ideally the average signalValue per base spread is between 100-1000.
    # strand - +/- to denote strand or orientation (whenever applicable). Use "." if no orientation is assigned.
    # signalValue - Measurement of overall (usually, average) enrichment for the region.
    # pValue - Measurement of statistical significance (-log10). Use -1 if no pValue is assigned.
    # qValue - Measurement of statistical significance using false discovery rate (-log10). Use -1 if no qValue is assigned.
    # peak - Point-source called for this peak; 0-based offset from chromStart. Use -1 if no point-source called.

    with open(saveto, "w") as stream:
        for p in peaks:
            center = sum(p.summit) // len(p.summit)
            stream.write(
                f"{p.contig}\t{p.start}\t{p.end}\t.\t0\t"
                f"{p.strand}\t{p.fe}\t{p.pvalue}\t{p.qvalue}\t{center - p.start}\n"
            )
