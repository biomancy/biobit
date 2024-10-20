import pickle

import pytest

from biobit.core.ngs import MatesOrientation


def test_mates_orientation():
    assert MatesOrientation.Inward == MatesOrientation(MatesOrientation.Inward) == MatesOrientation("I")

    with pytest.raises(ValueError):
        MatesOrientation("invalid")


def test_mates_orientation_pickle():
    assert MatesOrientation.Inward == pickle.loads(pickle.dumps(MatesOrientation.Inward)) == MatesOrientation("I")
