import pickle

from biobit.toolkit import repeto


def test_scoring():
    default = repeto.predict.Scoring()
    assert default.gap_open == -5
    assert default.gap_extend == -1
    assert default.complementary == 1
    assert default.mismatch == -2
    assert default == repeto.predict.Scoring()
    assert pickle.loads(pickle.dumps(default)) == default

    default.complementary = 100
    assert default.complementary == 100
    default.mismatch = -5
    assert default.mismatch == -5

    default.gap_open, default.gap_extend = 3, -100
    assert (default.gap_open, default.gap_extend) == (3, -100)


def test_filter():
    first = repeto.predict.Filter()
    second = repeto.predict.Filter()
    assert first == second == repeto.predict.Filter()
    assert pickle.loads(pickle.dumps(first)) == first

    for flter in first, second:
        flter \
            .set_min_score(100) \
            .set_rois([(0, 10), (0, 100), (110, 120)]) \
            .set_min_roi_overlap(3, 0) \
            .set_min_matches(0, 1000)

    assert first == second
    assert pickle.loads(pickle.dumps(first)) == first == second
    assert pickle.loads(pickle.dumps(second)) == first == second


def test_predict():
    ir, scores = repeto.predict.run(
        b"AANNUU", repeto.predict.Filter(), repeto.predict.Scoring()
    )
    irs = sorted(zip(scores, ir), key=lambda x: x[0])

    maxima = irs[-1]
    assert maxima[0] == 2
    assert maxima[1] == repeto.repeats.InvRepeat([repeto.repeats.InvSegment((0, 2), (4, 6))])
