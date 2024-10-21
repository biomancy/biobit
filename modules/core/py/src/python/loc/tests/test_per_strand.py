import pickle

from biobit.core.loc import PerStrand


def test_per_orientation_new():
    orient = PerStrand(11, -10)
    assert orient.forward == orient[1] == orient["+"] == 11
    assert orient.reverse == orient[-1] == orient["-"] == -10

    orient.forward = 2
    assert orient.forward == orient[1] == orient["+"] == 2


def test_per_orientation_pickle():
    assert pickle.loads(pickle.dumps(PerStrand(1, -12))) == PerStrand(1, -12)
