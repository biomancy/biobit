import pytest

from biobit_core_py.loc import Segment


def test_segment_new():
    assert Segment(0, 10) == Segment(0, 10)

    for start, end in (1, 0), (0, 0):
        with pytest.raises(ValueError):
            Segment(start, end)


def test_segment_len():
    assert Segment(0, 10).len() == 10
    assert Segment(0, 1).len() == 1


def test_segment_contains():
    segment = Segment(1, 10)
    assert segment.contains(0) is False
    assert segment.contains(1) is True
    assert segment.contains(5) is True
    assert segment.contains(9) is True
    assert segment.contains(10) is False
    assert segment.contains(11) is False


def test_intersects():
    segment = Segment(1, 10)
    assert segment.intersects(Segment(0, 1)) is False
    assert segment.intersects(Segment(0, 2)) is True
    assert segment.intersects(Segment(5, 9)) is True
    assert segment.intersects(Segment(9, 10)) is True
    assert segment.intersects(Segment(10, 11)) is False


def test_touches():
    segment = Segment(1, 10)
    assert segment.touches(Segment(0, 1)) is True
    assert segment.touches(Segment(0, 2)) is False
    assert segment.touches(Segment(5, 9)) is False
    assert segment.touches(Segment(9, 10)) is False
    assert segment.touches(Segment(10, 11)) is True


def test_extend():
    segment = Segment(1, 10)
    assert segment.extend(1, 2) is None and segment == Segment(0, 12)
    assert segment.extend(1, 0) is None and segment == Segment(-1, 12)

    segment.extend(right=100)
    assert segment == Segment(-1, 112)

    segment.extend(left=100)
    assert segment == Segment(-101, 112)


def test_extended():
    segment = Segment(1, 10)
    assert segment.extended(1, 2) == Segment(0, 12)
    assert segment.extended(1, 0) == Segment(0, 10)
    assert segment == Segment(1, 10)


def test_intersection():
    segment = Segment(1, 10)
    assert segment.intersection(Segment(0, 1)) is None
    assert segment.intersection(Segment(0, 2)) == Segment(1, 2)
    assert segment.intersection(Segment(5, 9)) == Segment(5, 9)
    assert segment.intersection(Segment(9, 11)) == Segment(9, 10)
    assert segment.intersection(Segment(10, 11)) is None


def test_union():
    segment = Segment(1, 10)
    assert segment.union(Segment(0, 1)) == Segment(0, 10)
    assert segment.union(Segment(0, 2)) == Segment(0, 10)
    assert segment.union(Segment(5, 9)) == Segment(1, 10)
    assert segment.union(Segment(9, 11)) == Segment(1, 11)
    assert segment.union(Segment(-1, 0)) is None
    assert segment.union(Segment(11, 12)) is None
