import os
from pathlib import Path
from typing import Callable

from biobit.toolkit import seqproj

from . import descriptor

__all__ = ["into_seqproj"]


def into_seqproj(
        project: seqproj.Project, resfolder: os.PathLike[str], *,
        seqexp2descriptor: Callable[[seqproj.Experiment], str] = descriptor.from_seqexp,
        ensure_exists: bool = False
) -> seqproj.Project:
    """
    Parse the nf-core/rnaseq pipeline results and annotate the source seqproj with the generated files.

    The following attributes are added to each seqproj.Experiment:
    - __nfcore_rnaseq_bam__: Path to the BAM file
    - __nfcore_rnaseq_resfolder__: Path to the results folder
    - __nfcore_rnaseq_bigwig__: Path to bigwig file (for unstranded libraries)
    - __nfcore_rnaseq_bigwig_{fwd,rev}__: Path to strand-specific bigwig files
    - __nfcore_rnaseq_salmon__: Path to the salmon expression file

    :param project: Source seqproj.Project used to generate the nf-core/rnaseq pipeline. Each experiment will be
                    annotated with the generated files.
    :param resfolder: Folder with the results of the nf-core/rnaseq pipeline
    :param seqexp2descriptor: Function that maps experiments to descriptors in the nf-core/rnaseq design file
    :param ensure_exists: If True, check that the generated files exist
    """
    resfolder = Path(resfolder)

    for exp in project.experiments:
        # Sanity check
        for attr in "bam", "bigwig_fwd", "bigwig_rev", "salmon":
            if f"__nfcore_rnaseq_{attr}__" in exp.attributes:
                raise ValueError(f"Attribute '__nfcore_rnaseq_{attr}__' already exists: {exp}")

        descriptor = seqexp2descriptor(exp)

        # Results folder
        exp.attributes["__nfcore_rnaseq_resfolder__"] = resfolder.as_posix()

        # BAM file
        bam = resfolder / "star_salmon" / f"{descriptor}.markdup.sorted.bam"
        exp.attributes["__nfcore_rnaseq_bam__"] = bam.as_posix()

        # Bigwig
        if exp.library.strandedness is None:
            raise ValueError(f"Experiments with unknown strandedness are not supported: {descriptor}")
        elif exp.library.strandedness == seqproj.Strandedness.Unstranded:
            bigwig = resfolder / "star_salmon" / "bigwig" / f"{descriptor}.unstranded.bigWig"
            exp.attributes["__nfcore_rnaseq_bigwig__"] = bigwig.as_posix()
        else:
            assert exp.library.strandedness in (seqproj.Strandedness.Forward, seqproj.Strandedness.Reverse)
            bigwig_fwd = resfolder / "star_salmon" / "bigwig" / f"{descriptor}.forward.bigWig"
            exp.attributes["__nfcore_rnaseq_bigwig_fwd__"] = bigwig_fwd.as_posix()

            bigwig_rev = resfolder / "star_salmon" / "bigwig" / f"{descriptor}.reverse.bigWig"
            exp.attributes["__nfcore_rnaseq_bigwig_rev__"] = bigwig_rev.as_posix()

        # Salmon file
        salmon = resfolder / "star_salmon" / descriptor / "quant.sf"
        exp.attributes["__nfcore_rnaseq_salmon__"] = salmon.as_posix()

        if ensure_exists:
            for file in [bam, bigwig_fwd, bigwig_rev, salmon]:
                if not file.is_file():
                    raise ValueError(f"File {file} does not exist")
    return project
