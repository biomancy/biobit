import pickle

from biobit.toolkit.reaper import postfilter


def test_reaper_postfilter():
    nms1 = postfilter.NMS() \
        .set_fecutoff(10) \
        .set_group_within(1) \
        .set_slopfrac(0.1) \
        .set_sloplim(1, 2) \
        .set_boundaries("+", [1, 2, 2, 3, 4]) \
        .set_boundaries("-", [5]) \
        .set_boundaries("=", [1, 2, 3])

    nms2 = postfilter.NMS() \
        .set_fecutoff(10) \
        .set_group_within(1) \
        .set_slopfrac(0.1) \
        .set_sloplim(1, 2) \
        .set_boundaries("+", [1, 2, 2, 3, 4, 4, 4, 4]) \
        .set_boundaries("-", [5, 5]) \
        .set_boundaries("=", [2, 2, 1, 1, 1, 2, 3, 3, 3, 3])

    assert nms1 == nms2

    assert pickle.loads(pickle.dumps(nms1)) == nms1 == nms2
