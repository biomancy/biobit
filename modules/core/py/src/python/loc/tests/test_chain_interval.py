import pickle

import pytest

from biobit.core.loc import ChainInterval, Interval


def test_interval_chain_new():
    load = [(0, 10), (20, 30)]
    chain = ChainInterval(load)
    assert chain == ChainInterval([(0, 10), (20, 30)]) == load

    _chain = list(chain)
    assert len(_chain) == len(load) == 2
    for i, (ch, exp) in enumerate(zip(_chain, load)):
        assert isinstance(ch, Interval)
        assert ch == exp
    assert _chain == load

    # Touching/Overlapping/Unsorted intervals are not allowed
    for load in [(0, 10), (10, 20)], [(0, 10), (5, 15)], [(0, 10), (20, 30), (15, 25)]:
        with pytest.raises(Exception):
            ChainInterval(load)

    # Single interval is allowed
    ChainInterval(((0, 10),))


def test_pickle_chain_interval():
    chain = ChainInterval([(0, 10), (20, 30)])
    assert pickle.loads(pickle.dumps(chain)) == chain
    assert pickle.loads(pickle.dumps(chain)) == ChainInterval([(0, 10), (20, 30)])
