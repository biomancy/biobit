import pytest

from biobit.core.loc import Interval
from biobit.deprecated.gindex.overlap import Overlap, OverlapSteps


@pytest.fixture
def overlap_instance():
    rng = Interval(0, 10)
    intervals = [Interval(1, 3), Interval(4, 6), Interval(7, 9)]
    annotations = ['a', 'b', 'c']
    return Overlap(rng, intervals, annotations)


@pytest.fixture
def overlap_steps_instance():
    rng = Interval(0, 10)
    boundaries = [Interval(0, 2), Interval(2, 4), Interval(4, 6), Interval(6, 8), Interval(8, 10)]
    annotations = [{'a'}, {'b'}, {'c'}, {'d'}, {'e'}]
    return OverlapSteps(rng, boundaries, annotations)


def test_overlap_len(overlap_instance):
    assert len(overlap_instance) == 3


def test_overlap_iter(overlap_instance):
    expected = [(Interval(1, 3), 'a'), (Interval(4, 6), 'b'), (Interval(7, 9), 'c')]
    assert list(overlap_instance) == expected


def test_overlap_to_steps(overlap_instance):
    steps = overlap_instance.to_steps()
    assert isinstance(steps, OverlapSteps)
    assert steps.rng == overlap_instance.rng
    assert steps.boundaries == [
        Interval(0, 1), Interval(1, 3), Interval(3, 4), Interval(4, 6), Interval(6, 7), Interval(7, 9), Interval(9, 10)
    ]
    assert steps.annotations == [set(), {'a'}, set(), {'b'}, set(), {'c'}, set()]


def test_overlap_steps_len(overlap_steps_instance):
    assert len(overlap_steps_instance) == 5


def test_overlap_steps_iter(overlap_steps_instance):
    expected = [
        (Interval(0, 2), {'a'}), (Interval(2, 4), {'b'}), (Interval(4, 6), {'c'}), (Interval(6, 8), {'d'}),
        (Interval(8, 10), {'e'})
    ]
    assert list(overlap_steps_instance) == expected


def test_overlap_to_steps_nested_intervals(overlap_instance):
    # Create an Overlap instance with nested intervals
    rng = Interval(0, 10)
    intervals = [Interval(1, 9), Interval(2, 8), Interval(3, 7)]
    annotations = ['a', 'b', 'c']
    overlap = Overlap(rng, intervals, annotations)

    # Call the to_steps method
    steps = overlap.to_steps()

    # Assert that the returned OverlapSteps instance has the correct rng, boundaries, and annotations
    assert isinstance(steps, OverlapSteps)
    assert steps.rng == overlap.rng
    assert steps.boundaries == [
        Interval(0, 1), Interval(1, 2), Interval(2, 3), Interval(3, 7), Interval(7, 8), Interval(8, 9), Interval(9, 10)
    ]
    assert steps.annotations == [set(), {'a'}, {'a', 'b'}, {'a', 'b', 'c'}, {'a', 'b'}, {'a'}, set()]


def test_overlap_empty_intervals(overlap_instance):
    # Create an Overlap instance with no intervals
    rng = Interval(0, 10)
    overlap = Overlap(rng, [], [])

    # Assert that the length is 0
    assert len(overlap) == 0

    # Assert that converting to steps results in a single boundary with no annotations
    steps = overlap.to_steps()
    assert len(steps) == 1
    assert steps.boundaries == [Interval(0, 10)]
    assert steps.annotations == [set()]


def test_overlap_steps_empty_boundaries(overlap_steps_instance):
    # Create an OverlapSteps instance with no boundaries
    rng = Interval(0, 10)
    boundaries = []
    annotations = []

    with pytest.raises(ValueError):
        OverlapSteps(rng, boundaries, annotations)


def test_overlap_to_steps_single_interval(overlap_instance):
    # Create an Overlap instance with a single interval
    rng = Interval(0, 10)
    intervals = [Interval(1, 9)]
    annotations = ['a']
    overlap = Overlap(rng, intervals, annotations)

    # Call the to_steps method
    steps = overlap.to_steps()

    # Assert that the returned OverlapSteps instance has the correct rng, boundaries, and annotations
    assert isinstance(steps, OverlapSteps)
    assert steps.rng == overlap.rng
    assert steps.boundaries == [
        Interval(0, 1), Interval(1, 9), Interval(9, 10)
    ]
    assert steps.annotations == [set(), {'a'}, set()]
