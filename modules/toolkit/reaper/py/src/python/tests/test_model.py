import copy
import pickle

from biobit.toolkit.reaper import model


def test_reaper_model():
    pileup1 = model.RNAPileup() \
        .set_sensitivity(0.5) \
        .set_min_signal(10) \
        .set_control_baseline(0.2) \
        .add_control_model("+", [[(1, 2), (4, 6)], [(4, 5)]], True, [10, 20]) \
        .add_control_model("-", [[(1, 2), (4, 6), (8, 9)]], False, [10, 20])
    pileup2 = model.RNAPileup() \
        .set_sensitivity(0.5) \
        .set_min_signal(10) \
        .set_control_baseline(0.2) \
        .add_control_model("-", [[(1, 2), (4, 6), (8, 9)]], False, [10, 20]) \
        .add_control_model("+", [[(1, 2), (4, 6)], [(4, 5)]], True, [10, 20])

    assert pileup1 == pileup2

    assert pickle.loads(pickle.dumps(pileup1)) == pileup1 == pileup2

    deepcopy = copy.deepcopy(pileup1)
    assert deepcopy == pileup1 and deepcopy is not pileup1

    deepcopy.add_control_model(
        "+", [[(1, 2), (400, 600)], [(4, 5)]], True, [10, 20]
    )
    assert deepcopy != pileup1 and pileup1 == pileup2
