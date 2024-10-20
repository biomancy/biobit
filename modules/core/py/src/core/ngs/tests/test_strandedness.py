import pickle

import pytest

from biobit.core.ngs import Strandedness


def test_strandedness():
    assert Strandedness.Forward == Strandedness(Strandedness.Forward) == Strandedness("F")
    assert Strandedness.Reverse == Strandedness(Strandedness.Reverse) == Strandedness("R")
    assert Strandedness.Unstranded == Strandedness(Strandedness.Unstranded) == Strandedness("U")

    with pytest.raises(ValueError):
        Strandedness("invalid")


def test_strandedness_pickle():
    assert Strandedness.Forward == pickle.loads(pickle.dumps(Strandedness.Forward)) == Strandedness("F")
    assert Strandedness.Reverse == pickle.loads(pickle.dumps(Strandedness.Reverse)) == Strandedness("R")
    assert Strandedness.Unstranded == pickle.loads(pickle.dumps(Strandedness.Unstranded)) == Strandedness("U")
