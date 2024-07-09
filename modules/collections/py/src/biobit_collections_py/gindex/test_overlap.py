import pytest

from biobit.ds.gindex.overlap import Overlap, OverlapSteps
from biobit.core import Range


@pytest.fixture
def overlap_instance():
    rng = Range(0, 10)
    intervals = [Range(1, 3), Range(4, 6), Range(7, 9)]
    annotations = ['a', 'b', 'c']
    return Overlap(rng, intervals, annotations)


@pytest.fixture
def overlap_steps_instance():
    rng = Range(0, 10)
    boundaries = [Range(0, 2), Range(2, 4), Range(4, 6), Range(6, 8), Range(8, 10)]
    annotations = [{'a'}, {'b'}, {'c'}, {'d'}, {'e'}]
    return OverlapSteps(rng, boundaries, annotations)


def test_overlap_len(overlap_instance):
    assert len(overlap_instance) == 3


def test_overlap_iter(overlap_instance):
    expected = [(Range(1, 3), 'a'), (Range(4, 6), 'b'), (Range(7, 9), 'c')]
    assert list(overlap_instance) == expected


def test_overlap_to_steps(overlap_instance):
    steps = overlap_instance.to_steps()
    assert isinstance(steps, OverlapSteps)
    assert steps.rng == overlap_instance.rng
    assert steps.boundaries == [
        Range(0, 1), Range(1, 3), Range(3, 4), Range(4, 6), Range(6, 7), Range(7, 9), Range(9, 10)
    ]
    assert steps.annotations == [set(), {'a'}, set(), {'b'}, set(), {'c'}, set()]


def test_overlap_steps_len(overlap_steps_instance):
    assert len(overlap_steps_instance) == 5


def test_overlap_steps_iter(overlap_steps_instance):
    expected = [
        (Range(0, 2), {'a'}), (Range(2, 4), {'b'}), (Range(4, 6), {'c'}), (Range(6, 8), {'d'}),
        (Range(8, 10), {'e'})
    ]
    assert list(overlap_steps_instance) == expected


def test_overlap_to_steps_nested_intervals(overlap_instance):
    # Create an Overlap instance with nested intervals
    rng = Range(0, 10)
    intervals = [Range(1, 9), Range(2, 8), Range(3, 7)]
    annotations = ['a', 'b', 'c']
    overlap = Overlap(rng, intervals, annotations)

    # Call the to_steps method
    steps = overlap.to_steps()

    # Assert that the returned OverlapSteps instance has the correct rng, boundaries, and annotations
    assert isinstance(steps, OverlapSteps)
    assert steps.rng == overlap.rng
    assert steps.boundaries == [
        Range(0, 1), Range(1, 2), Range(2, 3), Range(3, 7), Range(7, 8), Range(8, 9), Range(9, 10)
    ]
    assert steps.annotations == [set(), {'a'}, {'a', 'b'}, {'a', 'b', 'c'}, {'a', 'b'}, {'a'}, set()]


def test_overlap_empty_intervals(overlap_instance):
    # Create an Overlap instance with no intervals
    rng = Range(0, 10)
    overlap = Overlap(rng, [], [])

    # Assert that the length is 0
    assert len(overlap) == 0

    # Assert that converting to steps results in a single boundary with no annotations
    steps = overlap.to_steps()
    assert len(steps) == 1
    assert steps.boundaries == [Range(0, 10)]
    assert steps.annotations == [set()]


def test_overlap_steps_empty_boundaries(overlap_steps_instance):
    # Create an OverlapSteps instance with no boundaries
    rng = Range(0, 10)
    boundaries = []
    annotations = []

    with pytest.raises(ValueError):
        OverlapSteps(rng, boundaries, annotations)


def test_overlap_to_steps_single_interval(overlap_instance):
    # Create an Overlap instance with a single interval
    rng = Range(0, 10)
    intervals = [Range(1, 9)]
    annotations = ['a']
    overlap = Overlap(rng, intervals, annotations)

    # Call the to_steps method
    steps = overlap.to_steps()

    # Assert that the returned OverlapSteps instance has the correct rng, boundaries, and annotations
    assert isinstance(steps, OverlapSteps)
    assert steps.rng == overlap.rng
    assert steps.boundaries == [
        Range(0, 1), Range(1, 9), Range(9, 10)
    ]
    assert steps.annotations == [set(), {'a'}, set()]
