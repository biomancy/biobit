from itertools import chain
from typing import Iterator

from attrs import define, field

from biobit.core import Range, Orientation
from biobit.core.strdeductor import StrandDeductor
from biobit.io import bam
from .source import Source, AlignedBlocks


@define(frozen=True, slots=True)
class SingleEndBAM(Source):
    obj: bam.Reader
    strander: StrandDeductor | None = field(default=None)

    def fetch(self, contig: str, start: int, end: int) -> Iterator[AlignedBlocks]:
        for segment in self.obj.fetch(contig, start, end):
            if self.strander is None:
                orientation = Orientation.dual
            else:
                strand = self.strander(segment)
                orientation = Orientation.from_strand(strand)

            blocks = tuple(Range(start, end) for start, end in segment.get_blocks())
            yield AlignedBlocks(blocks, orientation)

    def stats(self) -> dict:
        stats = {}
        if self.obj.statistics:
            assert self.obj.discarded is not None and self.obj.consumed is not None
            stats.update({
                "Discarded reads": self.obj.discarded.total,
                "Processed reads": self.obj.consumed.total,
            })
        return stats


@define(frozen=True, slots=True)
class PairedEndBAM(Source):
    obj: bam.PEReadsBundler
    strander: StrandDeductor | None = field(default=None)

    def fetch(self, contig: str, start: int, end: int) -> Iterator[AlignedBlocks]:
        for lmate, rmate in self.obj.fetch(contig, start, end):
            if self.strander is None:
                orientation = Orientation.dual
            else:
                strand = self.strander(lmate)
                assert strand == self.strander(rmate)
                orientation = Orientation.from_strand(strand)

            blocks = tuple(
                Range(start, end) for start, end in set(chain(lmate.get_blocks(), rmate.get_blocks()))
            )
            yield AlignedBlocks(blocks, orientation)

    def stats(self) -> dict:
        stats = {
            "Unpaired reads": self.obj.unpaired
        }

        if self.obj.reader.statistics:
            assert self.obj.reader.discarded is not None and self.obj.reader.consumed is not None
            stats.update({
                "Discarded reads": self.obj.reader.discarded.total,
                "Processed reads": self.obj.reader.consumed.total,
            })
        return stats
