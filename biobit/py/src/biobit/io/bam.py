from collections import defaultdict
from pathlib import Path
from typing import Iterable, Self, Iterator

from attrs import define, field
from pysam import AlignedSegment, AlignmentFile


@define(slots=True)
class ConsumedReads:
    paired: int = 0
    proper_pair: int = 0
    unmapped: int = 0
    mate_unmapped: int = 0
    reverse_strand: int = 0
    mate_reverse_strand: int = 0
    first_in_pair: int = 0
    second_in_pair: int = 0
    not_primary_alignment: int = 0
    fails_quality_checks: int = 0
    duplicate: int = 0
    supplementary: int = 0
    total: int = 0

    def count(self, read: AlignedSegment):
        if read.is_paired:
            self.paired += 1
        if read.is_proper_pair:
            self.proper_pair += 1
        if read.is_unmapped:
            self.unmapped += 1
        if read.mate_is_unmapped:
            self.mate_unmapped += 1
        if read.is_reverse:
            self.reverse_strand += 1
        if read.mate_is_reverse:
            self.mate_reverse_strand += 1
        if read.is_read1:
            self.first_in_pair += 1
        if read.is_read2:
            self.second_in_pair += 1
        if read.is_secondary:
            self.not_primary_alignment += 1
        if read.is_qcfail:
            self.fails_quality_checks += 1
        if read.is_duplicate:
            self.duplicate += 1
        if read.is_supplementary:
            self.supplementary += 1
        self.total += 1

    def summarize(self, prefix: str = "") -> str:
        return (
            f"{prefix}Paired: {self.paired}({self.paired / self.total:.2%})\n"
            f"{prefix}Proper pair: {self.proper_pair}({self.proper_pair / self.total:.2%})\n"
            f"{prefix}Unmapped: {self.unmapped}({self.unmapped / self.total:.2%})\n"
            f"{prefix}Mate unmapped: {self.mate_unmapped}({self.mate_unmapped / self.total:.2%})\n"
            f"{prefix}Reverse strand: {self.reverse_strand}({self.reverse_strand / self.total:.2%})\n"
            f"{prefix}Mate reverse strand: {self.mate_reverse_strand}({self.mate_reverse_strand / self.total:.2%})\n"
            f"{prefix}First in pair: {self.first_in_pair}({self.first_in_pair / self.total:.2%})\n"
            f"{prefix}Second in pair: {self.second_in_pair}({self.second_in_pair / self.total:.2%})\n"
            f"{prefix}Not primary alignment: {self.not_primary_alignment}({self.not_primary_alignment / self.total:.2%})\n"
            f"{prefix}Fails quality checks: {self.fails_quality_checks}({self.fails_quality_checks / self.total:.2%})\n"
            f"{prefix}Duplicate: {self.duplicate}({self.duplicate / self.total:.2%})\n"
            f"{prefix}Supplementary: {self.supplementary}({self.supplementary / self.total:.2%})\n"
            f"{prefix}Total: {self.total}\n"
        )

    def __str__(self):
        return self.summarize()


class Reader:
    def __init__(self, filename: Path, inflags: int, exflags: int, minmapq: int, *, statistics: bool = False):
        self.filename: Path = filename
        self.sf: AlignmentFile | None = None
        self.iterator: Iterable[AlignedSegment] | None = None
        self.inflags: int = inflags
        self.exflags: int = exflags
        self.minmapq: int = minmapq

        self.statistics: bool = statistics
        self.consumed: ConsumedReads | None = None if not statistics else ConsumedReads()
        self.discarded: ConsumedReads | None = None if not statistics else ConsumedReads()

    def fetch(self, contig: str, start: int | None = None, end: int | None = None) -> Self:
        if self.sf is None:
            self.sf = AlignmentFile(self.filename.as_posix(), "rb")
        self.iterator = self.sf.fetch(contig, start, end)
        return self

    def _is_read_ok(self, segment: AlignedSegment) -> bool:
        return (
                segment.flag & self.inflags == self.inflags
                and segment.flag & self.exflags == 0
                and segment.mapping_quality >= self.minmapq
        )

    def _iter_with_statistics(self) -> Iterator[AlignedSegment]:
        assert self.consumed is not None and self.discarded is not None
        assert self.iterator is not None

        for segment in self.iterator:
            if self._is_read_ok(segment):
                self.consumed.count(segment)
                yield segment
            else:
                self.discarded.count(segment)

        self.iterator = None

    def _iter_without_statistics(self) -> Iterator[AlignedSegment]:
        assert self.iterator is not None
        for segment in self.iterator:
            if self._is_read_ok(segment):
                yield segment

        self.iterator = None

    def __iter__(self) -> Iterator[AlignedSegment]:
        if self.sf is None:
            self.sf = AlignmentFile(self.filename.as_posix(), "rb")
            self.iterator = self.sf

        if self.iterator is None:
            self.iterator = self.sf

        if self.statistics:
            return self._iter_with_statistics()
        else:
            return self._iter_without_statistics()

    def __deepcopy__(self, _):
        return Reader(self.filename, self.inflags, self.exflags, self.minmapq, statistics=self.statistics)

    def __getstate__(self):
        return self.filename, self.inflags, self.exflags, self.minmapq, self.statistics

    def __setstate__(self, state):
        self.filename, self.inflags, self.exflags, self.minmapq, self.statistics = state
        self.sf = None
        self.consumed = None if not self.statistics else ConsumedReads()
        self.discarded = None if not self.statistics else ConsumedReads()


@define(slots=True)
class _PEReadsCache:
    lmates: list[AlignedSegment] = field(factory=list)
    rmates: list[AlignedSegment] = field(factory=list)

    def bundle(self) -> Iterator[tuple[AlignedSegment, AlignedSegment]]:
        missed_lmates = []
        for lmate in self.lmates:
            mate_found = False
            for ind, rmate in enumerate(self.rmates):
                if (
                        lmate.next_reference_id == rmate.reference_id
                        and lmate.next_reference_start == rmate.reference_start
                        and lmate.mate_is_reverse == rmate.is_reverse
                        and lmate.mate_is_unmapped == rmate.is_unmapped
                        and rmate.next_reference_id == lmate.reference_id
                        and rmate.next_reference_start == lmate.reference_start
                        and rmate.mate_is_reverse == lmate.is_reverse
                        and rmate.mate_is_unmapped == lmate.is_unmapped
                ):
                    mate_found = True
                    break

            if not mate_found:
                missed_lmates.append(lmate)
            else:
                rmate = self.rmates.pop(ind)
                yield lmate, rmate

        self.lmates = missed_lmates

    def is_empty(self) -> bool:
        return len(self.lmates) == 0 and len(self.rmates) == 0


class PEReadsBundler:
    def __init__(self, reader: Reader, step: int = 100_000):
        self.reader: Reader = reader
        self.cache: dict[str, _PEReadsCache] = defaultdict(_PEReadsCache)

        self.step: int = step
        self.unpaired = 0

    def fetch(self, contig: str, start: int | None = None, end: int | None = None) -> Self:
        self.reader.fetch(contig, start, end)
        return self

    def __iter__(self) -> Iterator[tuple[AlignedSegment, AlignedSegment]]:
        for ind, segment in enumerate(self.reader):
            qname = segment.query_name
            assert qname is not None, "Query name must be present in the BAM file"

            cached = self.cache[qname]
            if segment.is_read1:
                cached.lmates.append(segment)
            else:
                cached.rmates.append(segment)

            if ind % self.step == 0 and ind > 0:
                for k, v in list(self.cache.items()):
                    yield from v.bundle()
                    if v.is_empty():
                        self.cache.pop(k)

        # Clean up the cache
        for k, v in list(self.cache.items()):
            yield from v.bundle()
            self.unpaired += len(v.lmates) + len(v.rmates)
        self.cache.clear()

    def summarize(self, prefix: str = "") -> str:
        return f"{prefix}Unpaired reads left: {self.unpaired}\n"
