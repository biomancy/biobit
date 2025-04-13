import pickle

import pytest

from biobit.collections.interval_tree import Hits, HitSegments, BatchHitSegments, BatchHits


def test_hit_segments():
    # Empty state
    empty: HitSegments[str] = HitSegments()
    assert len(empty) == 0
    assert list(empty) == []
    assert empty.segments() == []
    assert empty.data() == []
    assert empty == HitSegments() == pickle.loads(pickle.dumps(empty))

    # Some hits
    hits = Hits()
    hits.extend([
        ((10, 20), "A"), ((20, 30), "B"), ((25, 35), "C"), ((0, 15), "A"),
    ])

    query = [(0, 100), (150, 200)]
    expected = [
        [(0, 20), (20, 25), (25, 30), (30, 35), (35, 100), (150, 200)],
        [frozenset({"A"}), frozenset({"B"}), frozenset({"B", "C"}), frozenset({"C"}), frozenset({}), frozenset()],
    ]

    # Segment by creating a new HitSegments object on the fly
    segments = hits.segment(query)
    assert segments.segments() == expected[0]
    assert segments.data() == expected[1]
    assert len(segments) == 6
    assert list(segments) == list(zip(*expected))
    assert segments == pickle.loads(pickle.dumps(segments))

    # Segment into an existing HitSegments object
    segments2 = HitSegments()
    assert hits.segment(query, into=segments2) is segments2
    assert segments2 == segments
    assert segments2.segments() == expected[0]
    assert segments2.data() == expected[1]
    assert len(segments2) == 6
    assert list(segments2) == list(zip(*expected))

    # Test clearing
    segments.clear()
    assert len(segments) == 0
    assert list(segments) == []
    assert segments.segments() == []
    assert segments.data() == []
    assert segments == HitSegments() == pickle.loads(pickle.dumps(segments))

    # Test reusing
    assert hits.segment(query, into=segments) is segments
    assert segments == segments2


def test_batch_hit_segments():
    # Empty state
    empty: BatchHitSegments[str] = BatchHitSegments()
    assert len(empty) == 0
    assert list(empty) == []
    assert empty == BatchHitSegments() == pickle.loads(pickle.dumps(empty))

    with pytest.raises(IndexError):
        empty.segments(0)
    with pytest.raises(IndexError):
        empty.data(0)

    # Sample hits
    bhits = BatchHits()
    bhits.append([(10, 30), (20, 40)], ["A", "B"])
    bhits.append([(50, 60), (55, 65)], ["C", "D"])
    bhits.append([], [])

    queries = [
        [(20, 45)],
        [(50, 57), (58, 60), (70, 100)],
        [(70, 80), (90, 91)]
    ]

    # Expected results
    exp_q0_intervals = [(20, 30), (30, 40), (40, 45)]
    exp_q0_data_sets = [frozenset({"A", "B"}), frozenset({"B"}), frozenset()]

    exp_q1_intervals = [(50, 55), (55, 57), (58, 60), (70, 100)]
    exp_q1_data_sets = [frozenset({"C"}), frozenset({"C", "D"}), frozenset({"C", "D"}), frozenset()]

    exp_q2_intervals = [(70, 80), (90, 91)]
    exp_q2_data_sets = [frozenset(), frozenset()]

    expected = [
        (exp_q0_intervals, exp_q0_data_sets),
        (exp_q1_intervals, exp_q1_data_sets),
        (exp_q2_intervals, exp_q2_data_sets),
    ]

    # Segment by creating a new BatchHitSegments object on the fly
    on_the_fly: BatchHitSegments[str] = bhits.segment(queries)
    assert len(on_the_fly) == 3

    # Check results for each query individually
    assert on_the_fly.segments(0) == exp_q0_intervals
    assert on_the_fly.data(0) == exp_q0_data_sets
    assert on_the_fly.segments(1) == exp_q1_intervals
    assert on_the_fly.data(1) == exp_q1_data_sets
    assert on_the_fly.segments(2) == exp_q2_intervals
    assert on_the_fly.data(2) == exp_q2_data_sets

    with pytest.raises(IndexError):
        on_the_fly.segments(3)

    # Iterate over all queries in the batch
    assert list(on_the_fly) == expected

    # Check using cached buffer
    reused = bhits.segment(queries, into=empty)
    assert reused is empty
    assert list(reused) == expected
    assert reused == on_the_fly == pickle.loads(pickle.dumps(on_the_fly))

    # Clearing the object
    on_the_fly.clear()
    assert len(on_the_fly) == 0
    assert list(on_the_fly) == []
    assert on_the_fly == BatchHitSegments() == pickle.loads(pickle.dumps(on_the_fly))
    with pytest.raises(IndexError):
        on_the_fly.segments(0)

    # Test reusing the cleared object
    reused_on_the_fly = bhits.segment(queries, into=on_the_fly)
    assert reused_on_the_fly is on_the_fly
    assert list(reused_on_the_fly) == expected
    assert reused_on_the_fly == reused == pickle.loads(pickle.dumps(reused_on_the_fly))
