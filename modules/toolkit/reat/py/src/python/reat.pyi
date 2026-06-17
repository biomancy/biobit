from collections.abc import Sequence
from typing import Self

from biobit.core.ngs import Layout
from biobit.io.bam import IntoReader
from biobit.io.fasta import IndexedSources

from .result import SamplePileup
from .selection import Mismatches, RequiredOrMismatches, RequiredSites
from .task import Task

Selector = Mismatches | RequiredSites | RequiredOrMismatches

class Reat[T]:
    """
    Reference-aware RNA editing pileup runner.

    Notes:
    - Sample tags are compared with Python equality. Adding sources for an equal tag appends to
      that existing sample instead of replacing previous sources.
    - `run()` does not clear registered sources or tags. Use `reset()` to remove all samples.
    - BAM records are grouped by the supplied `Layout`; paired layouts currently require inward
      mate orientation.
    - The default selector is `Mismatches()` that returns all sites with at least one mismatch.
    """

    def __init__(
        self,
        reference: IndexedSources,
        selector: Selector | None = None,
        min_phred: int = 20,
        threads: int = -1,
    ) -> None:
        """
        Create a runner over indexed FASTA reference sources.

        `min_phred` filters low-quality read bases before counting.
        """
        ...

    def add_sources(
        self, tag: T, sources: Sequence[IntoReader], layout: Layout
    ) -> Self:
        """
        Register one or more BAM sources for `tag`.

        If `tag` compares equal to an existing tag, the new sources are merged with the existing
        source list. A later `run()` processes all registered sources for that sample.
        """
        ...

    def run(self, tasks: Sequence[Task]) -> list[SamplePileup[T]]:
        """
        Run REAT for all registered samples and supplied tasks.

        Results are returned in first-registration order for sample tags. Each
        `SamplePileup` groups task pileups by `(seqid, orientation)`.
        """
        ...

    def reset(self) -> Self:
        """Remove all registered tags and sources from this runner."""
        ...

    __hash__ = None  # type: ignore
