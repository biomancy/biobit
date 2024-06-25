import pytest

from biobit.core import Range
from biobit.core.range import OverlapInfo


def test_add_some():
    range1 = Range(1, 5)
    result = range1 + 2
    assert result.start == 3
    assert result.end == 7


def test_add_zero():
    range1 = Range(1, 5)
    result = range1 + 0
    assert result.start == 1
    assert result.end == 5


def test_sub():
    range1 = Range(1, 5)
    result = range1 - 1
    assert result.start == 0
    assert result.end == 4


def test_len():
    range1 = Range(1, 5)
    assert len(range1) == 4


def test_eq_true():
    range1 = Range(1, 5)
    assert range1 == Range(1, 5)


def test_eq_false():
    range1 = Range(1, 5)
    range2 = Range(3, 7)
    assert range1 != range2


def test_contains_true():
    range1 = Range(1, 5)
    range2 = Range(2, 4)
    assert range2 in range1


def test_contains_false():
    range1 = Range(1, 5)
    range2 = Range(5, 7)
    assert range2 not in range1


def test_overlap_some():
    range1 = Range(1, 5)
    range2 = Range(3, 7)
    overlap = range1.overlap(range2)
    assert isinstance(overlap, OverlapInfo)
    assert overlap.size == 2
    assert overlap.rng.start == 3
    assert overlap.rng.end == 5


def test_overlap_none():
    range1 = Range(1, 5)
    range2 = Range(6, 10)
    overlap = range1.overlap(range2)
    assert overlap is None


def test_overlap_none_touching():
    range1 = Range(1, 5)
    range2 = Range(5, 10)
    overlap = range1.overlap(range2)
    assert overlap is None


def test_overlap_with_start_end():
    range1 = Range(1, 5)
    overlap = range1.overlap(start=3, end=7)
    assert isinstance(overlap, OverlapInfo)
    assert overlap.size == 2
    assert overlap.rng.start == 3
    assert overlap.rng.end == 5


def test_overlap_without_other_or_start_end():
    range1 = Range(1, 5)
    with pytest.raises(ValueError):
        range1.overlap()


def test_add_with_non_int():
    range1 = Range(1, 5)
    with pytest.raises(TypeError):
        range1 + 'a'


def test_init_with_start_greater_than_end():
    with pytest.raises(ValueError):
        Range(5, 1)
