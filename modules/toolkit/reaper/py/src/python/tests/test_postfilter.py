import copy
import pickle

from biobit.toolkit.reaper import postfilter


def test_reaper_postfilter():
    nms1 = postfilter.NMS() \
        .set_fecutoff(10) \
        .set_group_within(1) \
        .set_slopfrac(0.1) \
        .set_sensitivity(0.5) \
        .set_sloplim(1, 2) \
        .add_regions("+", True, [[(1, 2), (4, 6)], [(4, 5)]]) \
        .add_regions("-", False, [[(1, 2), (4, 6), (8, 9)]]) \
        .add_regions("-", False, [[(100, 200), (300, 400)]])

    nms2 = postfilter.NMS() \
        .set_fecutoff(10) \
        .set_group_within(1) \
        .set_slopfrac(0.1) \
        .set_sensitivity(0.5) \
        .set_sloplim(1, 2) \
        .add_regions("-", False, [[(1, 2), (4, 6), (8, 9)]]) \
        .add_regions("+", True, [[(1, 2), (4, 6)], [(4, 5)]]) \
        .add_regions("-", False, [[(100, 200), (300, 400)]])

    assert nms1 == nms2

    assert pickle.loads(pickle.dumps(nms1)) == nms1 == nms2

    deepcopy = copy.deepcopy(nms1)
    assert deepcopy == nms1 and deepcopy is not nms1

    deepcopy.add_regions("+", True, [[(1, 2), (4, 6)]])
    assert deepcopy != nms1 and nms1 == nms2
