from typing import Iterator, Protocol

from attrs import define

from biobit.core import Range, Orientation


@define(slots=True, frozen=True)
class AlignedBlocks:
    blocks: tuple[Range, ...]
    orientation: Orientation


class Source(Protocol):
    """
    Abstraction that represents a source of reads. The source can be directed to fetch reads from a specific region
    and then iterated over to yield the reads in that region. Each read comes as a tuple of strand and a list of
    blocks (ranges) that denote the read's alignment to the reference genome.

    Sources are though to be cheap to clone, but not to be shared between threads.
    """

    def fetch(self, contig: str, start: int, end: int) -> Iterator[AlignedBlocks]:
        """
        Fetch reads from a specific region of the reference genome.
        :param contig: The contig to fetch reads from.
        :param start: The start position of the region.
        :param end: The end position of the region.
        :return: Iterator of read blocks in the region.
        """
        pass

    def stats(self) -> dict:
        """
        Return statistics about processed reads.
        :return: A dictionary containing statistics about the source. The content is implementation-specific.
        """
        pass
