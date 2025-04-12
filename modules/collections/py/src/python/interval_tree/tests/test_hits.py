import pickle

import pytest

from biobit.collections.interval_tree import Hits, BatchHits


def test_hits():
    empty: Hits[str] = Hits()
    assert len(empty) == 0 and list(empty) == []
    assert empty.intervals() == [] and empty.data() == []
    assert Hits() == empty
    assert pickle.loads(pickle.dumps(empty)) == empty

    segments = [(-10, 10), (5, 10), (0, 5)]
    elements = ["a", "b", "a"]

    hits = Hits()
    hits.extend(zip(segments, elements))

    assert len(hits) == 3 and list(hits) == list(zip(segments, elements))
    assert hits.intervals() == segments and hits.data() == elements

    hits.append((0, 10), "c")
    assert len(hits) == 4 and list(hits) == list(zip(segments, elements)) + [((0, 10), "c")]

    other = Hits()
    other.extend(zip(segments + [(0, 10)], elements + ["c"]))
    assert other == hits == pickle.loads(pickle.dumps(hits))

    hits.clear()
    assert len(hits) == 0 and list(hits) == []
    assert hits == Hits()


def test_batch_hits():
    empty: BatchHits[str] = BatchHits()
    assert len(empty) == 0  # 0 queries
    assert list(empty) == []
    assert empty == BatchHits() == pickle.loads(pickle.dumps(empty))

    with pytest.raises(IndexError):
        empty.intervals(0)
    with pytest.raises(IndexError):
        empty.data(0)

    # Sample batch with one query
    bhits = BatchHits()
    bhits.append(
        [(-10, 10), (5, 10)],
        ["a", "b"],
    )
    assert len(bhits) == 1
    assert bhits.intervals(0) == [(-10, 10), (5, 10)]
    assert bhits.data(0) == ["a", "b"]
    assert list(bhits) == [([(-10, 10), (5, 10)], ["a", "b"])]
    assert bhits == pickle.loads(pickle.dumps(bhits))

    with pytest.raises(IndexError):
        bhits.intervals(1)
    with pytest.raises(IndexError):
        bhits.data(1)

    # Multiple queries
    bhits.extend([
        ([(0, 5), (10, 13)], ["a", "C"]),
        ([(0, 10), (20, 30)], ["c", "d"]),
    ])

    assert len(bhits) == 3
    assert bhits.intervals(0) == [(-10, 10), (5, 10)]
    assert bhits.data(0) == ["a", "b"]
    assert bhits.intervals(1) == [(0, 5), (10, 13)]
    assert bhits.data(1) == ["a", "C"]
    assert bhits.intervals(2) == [(0, 10), (20, 30)]
    assert bhits.data(2) == ["c", "d"]
    assert list(bhits) == [
        ([(-10, 10), (5, 10)], ["a", "b"]),
        ([(0, 5), (10, 13)], ["a", "C"]),
        ([(0, 10), (20, 30)], ["c", "d"]),
    ]
    assert bhits == pickle.loads(pickle.dumps(bhits))

    with pytest.raises(IndexError):
        bhits.intervals(3)
    with pytest.raises(IndexError):
        bhits.data(3)

    # Clearing
    bhits.clear()
    assert len(bhits) == 0
    assert list(bhits) == []
    assert bhits == BatchHits() == pickle.loads(pickle.dumps(bhits))

    with pytest.raises(IndexError):
        bhits.intervals(0)
    with pytest.raises(IndexError):
        bhits.data(0)

    # Errors in append/extend
    for intervals, data in [
        ([], ["A"]), ([(0, 10), (1, 2)], []), ([None], [None])
    ]:
        with pytest.raises(ValueError):
            bhits.append(intervals, data)
        assert len(bhits) == 0
        assert list(bhits) == []
        assert bhits == BatchHits() == pickle.loads(pickle.dumps(bhits))

    with pytest.raises(ValueError):
        bhits.extend([
            ([(0, 10), (1, 2)], ["A"]),
            ([(3, 4)], []),
            ([None], [None])
        ])
    assert len(bhits) == 0
    assert list(bhits) == []
    assert bhits == BatchHits() == pickle.loads(pickle.dumps(bhits))
