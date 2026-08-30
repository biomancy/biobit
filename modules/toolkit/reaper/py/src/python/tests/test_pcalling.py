import pickle

import pytest

from biobit.toolkit.reaper import pcalling


def test_reaper_pcalling():
    cutoff1 = pcalling.ByCutoff().set_cutoff(10).set_min_length(2).set_merge_within(1)
    cutoff2 = pcalling.ByCutoff().set_cutoff(10).set_min_length(2).set_merge_within(1)

    assert cutoff1 == cutoff2

    assert pickle.loads(pickle.dumps(cutoff1)) == cutoff1 == cutoff2


def test_reaper_pcalling_rejects_invalid_numerical_settings():
    with pytest.raises(RuntimeError):
        pcalling.ByCutoff().set_min_length(0)
    with pytest.raises(RuntimeError):
        pcalling.ByCutoff().set_cutoff(float("nan"))
