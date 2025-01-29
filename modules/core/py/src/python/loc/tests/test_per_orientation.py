import pickle

from biobit.core.loc import PerOrientation


def test_per_orientation_new():
    orient = PerOrientation(1, -1, 0)
    assert orient.forward == orient[1] == orient["+"] == 1
    assert orient.reverse == orient[-1] == orient["-"] == -1
    assert orient.dual == orient[0] == orient["="] == 0

    orient.forward = 2
    assert orient.forward == orient[1] == orient["+"] == 2


def test_per_orientation_pickle():
    assert pickle.loads(pickle.dumps(PerOrientation(1, -12, 0))) == PerOrientation(1, -12, 0)
