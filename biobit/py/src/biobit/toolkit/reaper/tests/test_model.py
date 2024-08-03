import pickle

from biobit.toolkit.reaper import model


def test_reaper_model():
    pileup1 = model.RNAPileup().set_sensitivity(0.5).set_min_signal(10).set_control_baseline(0.2)
    pileup2 = model.RNAPileup().set_sensitivity(0.5).set_min_signal(10).set_control_baseline(0.2)

    assert pileup1 == pileup2

    assert pickle.loads(pickle.dumps(pileup1)) == pileup1 == pileup2
