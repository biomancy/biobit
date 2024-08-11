import pytest

from biobit.core.loc import Segment
from biobit.deprecated.gindex.overlap import Overlap, OverlapSteps


@pytest.fixture
def overlap_instance():
    rng = Segment(0, 10)
    intervals = [Segment(1, 3), Segment(4, 6), Segment(7, 9)]
    annotations = ['a', 'b', 'c']
    return Overlap(rng, intervals, annotations)


@pytest.fixture
def overlap_steps_instance():
    rng = Segment(0, 10)
    boundaries = [Segment(0, 2), Segment(2, 4), Segment(4, 6), Segment(6, 8), Segment(8, 10)]
    annotations = [{'a'}, {'b'}, {'c'}, {'d'}, {'e'}]
    return OverlapSteps(rng, boundaries, annotations)


def test_overlap_len(overlap_instance):
    assert len(overlap_instance) == 3


def test_overlap_iter(overlap_instance):
    expected = [(Segment(1, 3), 'a'), (Segment(4, 6), 'b'), (Segment(7, 9), 'c')]
    assert list(overlap_instance) == expected


def test_overlap_to_steps(overlap_instance):
    steps = overlap_instance.to_steps()
    assert isinstance(steps, OverlapSteps)
    assert steps.rng == overlap_instance.rng
    assert steps.boundaries == [
        Segment(0, 1), Segment(1, 3), Segment(3, 4), Segment(4, 6), Segment(6, 7), Segment(7, 9), Segment(9, 10)
    ]
    assert steps.annotations == [set(), {'a'}, set(), {'b'}, set(), {'c'}, set()]


def test_overlap_steps_len(overlap_steps_instance):
    assert len(overlap_steps_instance) == 5


def test_overlap_steps_iter(overlap_steps_instance):
    expected = [
        (Segment(0, 2), {'a'}), (Segment(2, 4), {'b'}), (Segment(4, 6), {'c'}), (Segment(6, 8), {'d'}),
        (Segment(8, 10), {'e'})
    ]
    assert list(overlap_steps_instance) == expected


def test_overlap_to_steps_nested_intervals(overlap_instance):
    # Create an Overlap instance with nested intervals
    rng = Segment(0, 10)
    intervals = [Segment(1, 9), Segment(2, 8), Segment(3, 7)]
    annotations = ['a', 'b', 'c']
    overlap = Overlap(rng, intervals, annotations)

    # Call the to_steps method
    steps = overlap.to_steps()

    # Assert that the returned OverlapSteps instance has the correct rng, boundaries, and annotations
    assert isinstance(steps, OverlapSteps)
    assert steps.rng == overlap.rng
    assert steps.boundaries == [
        Segment(0, 1), Segment(1, 2), Segment(2, 3), Segment(3, 7), Segment(7, 8), Segment(8, 9), Segment(9, 10)
    ]
    assert steps.annotations == [set(), {'a'}, {'a', 'b'}, {'a', 'b', 'c'}, {'a', 'b'}, {'a'}, set()]


def test_overlap_empty_intervals(overlap_instance):
    # Create an Overlap instance with no intervals
    rng = Segment(0, 10)
    overlap = Overlap(rng, [], [])

    # Assert that the length is 0
    assert len(overlap) == 0

    # Assert that converting to steps results in a single boundary with no annotations
    steps = overlap.to_steps()
    assert len(steps) == 1
    assert steps.boundaries == [Segment(0, 10)]
    assert steps.annotations == [set()]


def test_overlap_steps_empty_boundaries(overlap_steps_instance):
    # Create an OverlapSteps instance with no boundaries
    rng = Segment(0, 10)
    boundaries = []
    annotations = []

    with pytest.raises(ValueError):
        OverlapSteps(rng, boundaries, annotations)


def test_overlap_to_steps_single_interval(overlap_instance):
    # Create an Overlap instance with a single interval
    rng = Segment(0, 10)
    intervals = [Segment(1, 9)]
    annotations = ['a']
    overlap = Overlap(rng, intervals, annotations)

    # Call the to_steps method
    steps = overlap.to_steps()

    # Assert that the returned OverlapSteps instance has the correct rng, boundaries, and annotations
    assert isinstance(steps, OverlapSteps)
    assert steps.rng == overlap.rng
    assert steps.boundaries == [
        Segment(0, 1), Segment(1, 9), Segment(9, 10)
    ]
    assert steps.annotations == [set(), {'a'}, set()]
