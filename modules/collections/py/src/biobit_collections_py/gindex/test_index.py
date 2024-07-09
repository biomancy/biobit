import intervaltree as it
import pytest

from biobit.ds.gindex import GenomicIndex
from biobit.core import Orientation, Range


@pytest.fixture
def empty_index():
    return GenomicIndex()


@pytest.fixture
def complex_index():
    itrees = {}

    cache = it.IntervalTree()
    cache.addi(1, 10, data='1:10')
    cache.addi(20, 30, data='20:30')
    itrees[('1', Orientation.fwd)] = cache

    cache = it.IntervalTree()
    cache.addi(-5, 2, data='-5:2')
    cache.addi(0, 150, data='0:150')
    cache.addi(200, 300, data='200:300')
    itrees[('2', Orientation.fwd)] = cache

    cache = it.IntervalTree()
    cache.addi(0, 5, data='0:5')
    cache.addi(3, 10, data='3:10')
    itrees[('2', Orientation.rev)] = cache

    return GenomicIndex(itrees)


@pytest.fixture
def simple_index():
    cache = it.IntervalTree()
    cache.addi(-100, 100, data='-100:100')
    itrees = {('1', Orientation.fwd): cache}
    return GenomicIndex(itrees)


def test_merge(empty_index, simple_index, complex_index):
    merged = GenomicIndex.merge(empty_index, simple_index, complex_index)
    assert len(merged.itrees) == 3
    assert len(merged.itrees[('1', Orientation.fwd)]) == 3
    assert len(merged.itrees[('2', Orientation.fwd)]) == 3
    assert len(merged.itrees[('2', Orientation.rev)]) == 2

    overlap = merged.overlap('1', Orientation.fwd, 0, 10)
    assert overlap.intervals == [Range(0, 10), Range(1, 10)]
    assert overlap.annotations == ['-100:100', '1:10']

    overlap = merged.overlap('2', Orientation.fwd, -3, 10)
    assert overlap.intervals == [Range(-3, 2), Range(0, 10)]
    assert overlap.annotations == ['-5:2', '0:150']

    overlap = merged.overlap('2', Orientation.rev, -1, 8)
    assert overlap.intervals == [Range(0, 5), Range(3, 8)]
    assert overlap.annotations == ['0:5', '3:10']


def test_overlap_both_range_and_start_end_provided(empty_index):
    with pytest.raises(ValueError):
        empty_index.overlap('1', Orientation.fwd, 0, 10, Range(0, 10))


def test_overlap_no_range_provided(empty_index):
    with pytest.raises(ValueError):
        empty_index.overlap('1', Orientation.fwd)


def test_overlap_no_hits(complex_index):
    overlap = complex_index.overlap('1', Orientation.fwd, 100, 200)
    assert overlap.intervals == []
    assert overlap.annotations == []


@pytest.fixture
def bed_file(tmp_path):
    data = """\
chr1	0	1000	gene1	0	+
chr1	1000	2000	gene2	0	-
chr2	0	500	gene3	0	+
chr2	500	1000	gene4	0	-
"""
    bed_file_path = tmp_path / "test.bed"
    bed_file_path.write_text(data)
    return bed_file_path


def test_from_bed_no_filter(bed_file):
    index = GenomicIndex.from_bed(bed_file)

    overlap = index.overlap('chr1', Orientation.fwd, 0, 1000)
    assert overlap.intervals == [Range(0, 1000)]
    assert overlap.annotations == ['gene1']

    overlap = index.overlap('chr1', Orientation.rev, 1000, 1500)
    assert overlap.intervals == [Range(1000, 1500)]
    assert overlap.annotations == ['gene2']

    overlap = index.overlap('chr2', Orientation.fwd, 0, 1000)
    assert overlap.intervals == [Range(0, 500)]
    assert overlap.annotations == ['gene3']

    overlap = index.overlap('chr2', Orientation.rev, 500, 1500)
    assert overlap.intervals == [Range(500, 1000)]
    assert overlap.annotations == ['gene4']


def test_from_bed_with_filter(bed_file):
    index = GenomicIndex.from_bed(bed_file, filterfn=lambda it: it.name in {"gene1", "ABC"})

    overlap = index.overlap('chr1', Orientation.fwd, 940, 10000)
    assert overlap.intervals == [Range(940, 1000)]
    assert overlap.annotations == ['gene1']

    for contig, strand in ('chr1', Orientation.rev), ('chr2', Orientation.fwd), ('chr2', Orientation.rev):
        overlap = index.overlap(contig, strand, 0, 10000)
        assert overlap.intervals == []
        assert overlap.annotations == []
