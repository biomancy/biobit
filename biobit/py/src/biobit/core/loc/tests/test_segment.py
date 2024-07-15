import pytest

from biobit.core.loc import Segment


def test_segment_new():
    segment = Segment(0, 10)
    assert segment == Segment(0, 10) == (0, 10)
    assert segment.start == 0
    assert segment.end == 10

    with pytest.raises(Exception):
        segment.start = 13

    with pytest.raises(AttributeError):
        segment.end = 13

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

    for target, expected in [
        ((0, 1), False),
        ((0, 2), True),
        ((5, 9), True),
        ((9, 10), True),
        ((10, 11), False),
    ]:
        assert segment.intersects(target) is expected
        assert segment.intersects(Segment(*target)) is expected


def test_touches():
    segment = Segment(1, 10)

    for target, expected in [
        ((0, 1), True),
        ((0, 2), False),
        ((5, 9), False),
        ((9, 10), False),
        ((10, 11), True),
    ]:
        assert segment.touches(target) is expected
        assert segment.touches(Segment(*target)) is expected


def test_extend():
    segment = Segment(1, 10)
    assert segment.extend(1, 2) is segment and segment == Segment(0, 12)
    assert segment.extend(1, 0) is segment and segment == Segment(-1, 12)

    assert segment.extend(right=100) is segment and segment == Segment(-1, 112)
    assert segment.extend(left=100) is segment and segment == Segment(-101, 112)


def test_extended():
    segment = Segment(1, 10)
    assert segment.extended(1, 2) == Segment(0, 12)
    assert segment.extended(1, 0) == Segment(0, 10)
    assert segment == Segment(1, 10)


def test_intersection():
    segment = Segment(1, 10)

    for target in (0, 1), (10, 11):
        assert segment.intersection(target) is None
        assert segment.intersection(Segment(*target)) is None

    for target, expected in [
        ((0, 2), (1, 2)),
        ((5, 9), (5, 9)),
        ((9, 11), (9, 10)),
    ]:
        assert segment.intersection(target) == expected
        assert segment.intersection(target) == Segment(*expected)
        assert segment.intersection(Segment(*target)) == expected
        assert segment.intersection(Segment(*target)) == Segment(*expected)


def test_union():
    segment = Segment(1, 10)

    for target in (-1, 0), (11, 12):
        assert segment.union(target) is None
        assert segment.union(Segment(*target)) is None

    for target, expected in [
        ((0, 1), (0, 10)),
        ((0, 2), (0, 10)),
        ((5, 9), (1, 10)),
        ((9, 11), (1, 11)),
    ]:
        assert segment.union(target) == expected
        assert segment.union(target) == Segment(*expected)
        assert segment.union(Segment(*target)) == expected
        assert segment.union(Segment(*target)) == Segment(*expected)


def test_merge():
    assert Segment.merge([]) == []

    segments = [Segment(1, 10), (5, 15), Segment(20, 30)]
    assert Segment.merge(segments) == [Segment(1, 15), Segment(20, 30)]
