import pickle

from biobit.core.loc import ChainInterval
from biobit.core.loc.mapping import ChainMap


def test_chain_map_new():
    for load in [
        [(0, 10), ],
        [(0, 10), (20, 30), (40, 50)],
    ]:
        chain = ChainInterval(load)
        assert ChainMap(load) == ChainMap(chain)


def test_chain_map_mapping():
    chain = ChainMap([(0, 10), (20, 30)])

    assert chain.map_interval((0, 10)) == (0, 10)
    assert chain.map_interval((0, 25)) == (0, 15)
    assert chain.map_interval((-10, 0)) is None
    assert chain.map_interval((10, 20)) is None

    assert chain.invmap_interval((0, 10)) == [(0, 10)]
    assert chain.invmap_interval((0, 15)) == [(0, 10), (20, 25)]
    assert chain.invmap_interval((0, 25)) == [(0, 10), (20, 30)]
    assert chain.invmap_interval((-10, 0)) is None
    assert chain.invmap_interval((20, 30)) is None


def test_chain_map_pickle():
    chain = ChainInterval([(0, 10), (20, 30)])
    cmap = ChainMap(chain)
    assert pickle.loads(pickle.dumps(cmap)) == cmap
