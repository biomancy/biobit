import pickle

import pytest

from biobit.collections.interval_tree import Hits, BatchHits, Bits, BitsBuilder
from biobit.core.loc import Interval

DATA_A = "A"
DATA_B = "B"
DATA_C = "A"
DATA_D = "D"


# Fixture for a simple Bits tree
@pytest.fixture
def bits() -> Bits[str]:
    builder: BitsBuilder[str] = Bits.builder()
    builder.add(Interval(10, 20), DATA_A)
    builder.add((15, 25), DATA_B)
    builder.add((30, 40), DATA_C)
    builder.add(Interval(50, 60), DATA_D)  # Disconnected interval
    return builder.build()


def test_bits_building(bits: Bits[str]):
    builder: BitsBuilder[str] = Bits.builder()
    builder.extend([
        ((10, 20), DATA_A),
        (Interval(15, 25), DATA_B),
        ((30, 40), DATA_C),
        (Interval(50, 60), DATA_D)
    ])
    tree = builder.build()

    # Building from the same builder twice should yield empty trees
    empty = builder.build()
    assert not empty.records()
    assert not empty.data()
    assert not empty.intervals()

    # fixture and the builder should yield the same tree
    assert tree == bits
    assert tree.records() == bits.records() == [
        ((10, 20), DATA_A),
        ((15, 25), DATA_B),
        ((30, 40), DATA_C),
        ((50, 60), DATA_D),
    ]
    assert tree.data() == bits.data() == [DATA_A, DATA_B, DATA_C, DATA_D]
    assert tree.intervals() == bits.intervals() == [(10, 20), (15, 25), (30, 40), (50, 60)]


def test_bits_pickling(bits: Bits[str]):
    assert bits == pickle.loads(pickle.dumps(bits))


def test_bits_intersection(bits: Bits[str]):
    hits = bits.intersect_interval((12, 14))
    assert set(hits) == {
        (Interval(10, 20), DATA_A),
    }
    assert hits.intervals() == [Interval(10, 20)]
    assert hits.data() == [DATA_A]

    # Supply an external buffer
    buffer = Hits()
    buf = bits.intersect_interval((12, 14), into=buffer)
    assert buffer is buf and buf == hits

    # Reuse the buffer
    hits = bits.intersect_interval((10, 100), into=hits)
    assert set(hits) == set(bits.records())

    # Empty intersection
    assert bits.intersect_interval((0, 5)) == Hits()


def test_bits_batch_intersection(bits: Bits[str]):
    bhits = bits.batch_intersect_intervals([(12, 14), (18, 22), (55, 65)])
    assert len(bhits) == 3
    assert bhits.intervals(0) == [Interval(10, 20)]
    assert bhits.data(0) == [DATA_A]
    assert bhits.intervals(1) == [Interval(15, 25)]
    assert bhits.data(1) == [DATA_B]
    assert bhits.intervals(2) == [Interval(50, 60)]
    assert bhits.data(2) == [DATA_D]

    with pytest.raises(IndexError):
        bhits.intervals(3)

    assert list(bhits) == [
        ([Interval(10, 20)], [DATA_A]),
        ([Interval(15, 25)], [DATA_B]),
        ([Interval(50, 60)], [DATA_D]),
    ]

    # Supply an external buffer
    buffer = BatchHits()
    buf = bits.batch_intersect_intervals([(12, 14), (18, 22), (55, 65)], into=buffer)
    assert buffer is buf and buf == bhits

    # Reuse the buffer
    bhits = bits.batch_intersect_intervals([(10, 100)], into=bhits)
    assert list(bhits) == [(
        [(10, 20), (15, 25), (30, 40), (50, 60)],
        [DATA_A, DATA_B, DATA_C, DATA_D]
    )]

    # Empty intersection
    assert bits.batch_intersect_intervals([(0, 5)]) == BatchHits()

    # Empty batch
    assert bits.batch_intersect_intervals([]) == BatchHits()
