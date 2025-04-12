import pickle

from biobit.collections.interval_tree import Bits, BitsBuilder, Forest


def test_forest():
    forest = Forest[str, str]()
    assert len(forest) == 0
    assert list(forest.items()) == list(forest.keys()) == list(forest.values()) == list(forest) == []
    assert forest == pickle.loads(pickle.dumps(forest))

    # Forests can be modified in place
    forest["chr1"] = Bits[str].builder().build()
    assert len(forest) == 1
    assert list(forest.items()) == [("chr1", forest["chr1"])]

    del forest["chr1"]
    assert len(forest) == 0
    assert list(forest.items()) == []
    assert forest == Forest()

    # Identical forests are equal
    forest['1'] = BitsBuilder().add((1, 2), "a").build()
    forest['2'] = BitsBuilder().add((3, 4), "b").build()

    another_forest = Forest()
    another_forest['1'] = BitsBuilder().add((1, 2), "a").build()
    another_forest['2'] = BitsBuilder().add((3, 4), "b").build()

    assert forest == another_forest == pickle.loads(pickle.dumps(forest))
