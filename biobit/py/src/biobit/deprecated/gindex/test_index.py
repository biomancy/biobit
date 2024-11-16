import intervaltree as it
import pytest

from biobit.core.loc import Orientation, Interval
from biobit.deprecated.gindex.genomic_index import Bundle


@pytest.fixture
def empty_index():
    return Bundle()


@pytest.fixture
def complex_index():
    itrees = {}

    cache = it.IntervalTree()
    cache.addi(1, 10, data='1:10')
    cache.addi(20, 30, data='20:30')
    itrees[('1', Orientation.Forward)] = cache

    cache = it.IntervalTree()
    cache.addi(-5, 2, data='-5:2')
    cache.addi(0, 150, data='0:150')
    cache.addi(200, 300, data='200:300')
    itrees[('2', Orientation.Forward)] = cache

    cache = it.IntervalTree()
    cache.addi(0, 5, data='0:5')
    cache.addi(3, 10, data='3:10')
    itrees[('2', Orientation.Reverse)] = cache

    return Bundle(itrees)


@pytest.fixture
def simple_index():
    cache = it.IntervalTree()
    cache.addi(-100, 100, data='-100:100')
    itrees = {('1', Orientation.Forward): cache}
    return Bundle(itrees)


def test_overlap_both_range_and_start_end_provided(empty_index):
    with pytest.raises(ValueError):
        empty_index.overlap('1', Orientation.Forward, 0, 10, Interval(0, 10))


def test_overlap_no_range_provided(empty_index):
    with pytest.raises(ValueError):
        empty_index.overlap('1', Orientation.Forward)


def test_overlap_no_hits(complex_index):
    overlap = complex_index.overlap('1', Orientation.Forward, 100, 200)
    assert overlap.intervals == []
    assert overlap.annotations == []
