import os
from io import TextIOBase
from typing import Callable

from biobit.core.ngs import Strandedness
from biobit.toolkit import seqproj
from . import descriptor

__all__ = ["from_seqproj"]


def from_seqproj(
        project: seqproj.Project, saveto: os.PathLike[str] | TextIOBase | None, *,
        seqexp2desc: Callable[[seqproj.Experiment], str] = descriptor.from_seqexp
) -> str:
    """
    Converts a given seqproj into an input file (design) for the nf-core/rnaseq pipeline.

    The function generates a CSV table with columns: "sample", "fastq_1", "fastq_2", "strandedness".
    Each row corresponds to a run in the project's experiments. The "sample" column uses the experiment ID as the sample name.
    The "strandedness" column is derived from the experiment's library stranding, with "auto" used for unknown stranding.
    The "fastq_1" and "fastq_2" columns are populated with the file paths from the run's files.
    For single-end runs, "fastq_2" is left empty.

    :param project: The project object to be converted.
    :param saveto: The destination for the output. This can be a file path (as a string or Path object) or a TextIO stream.
    Use None to skip saving and return the content as a string.
    :param seqexp2desc: A function that converts a seqproj experiment into a human-readable descriptor (sample column).
    :return: The content of the generated input file.
    """
    # Columns
    lines = ["sample,fastq_1,fastq_2,strandedness"]
    for exp in project.experiments:
        descriptor = seqexp2desc(exp)

        # Convert stranding to the nf-core/rnaseq format
        match exp.library.strandedness:
            case None:
                stranding = "auto"
            case other:
                stranding = {
                    Strandedness.Forward: "forward",
                    Strandedness.Reverse: "reverse",
                    Strandedness.Unstranded: "unstranded"
                }[other]

        for run in exp.runs:
            if isinstance(run.layout, seqproj.Layout.Paired):
                fastq1, fastq2 = run.layout.files
            elif isinstance(run.layout, seqproj.Layout.Single):
                fastq1, fastq2 = run.layout.file, ""
            else:
                raise ValueError(f"Unsupported sequencing layout: {run.layout}")

            lines.append(f"{descriptor},{fastq1},{fastq2},{stranding}")

    content = "\n".join(lines)

    # Save & return the content
    if saveto:
        if isinstance(saveto, TextIOBase):
            saveto.write(content)
        else:
            with open(saveto, "w") as stream:
                stream.write(content)
    return content
