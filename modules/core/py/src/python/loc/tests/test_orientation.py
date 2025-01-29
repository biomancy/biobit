import pickle

import pytest

from biobit.core.loc import Orientation


def test_orientation_new():
    assert Orientation(1) == Orientation("+") == Orientation.Forward
    assert Orientation.Forward == "+" and Orientation.Forward == 1

    assert Orientation(-1) == Orientation("-") == Orientation.Reverse
    assert Orientation.Reverse == "-" and Orientation.Reverse == -1

    assert Orientation(0) == Orientation("=") == Orientation.Dual
    assert Orientation.Dual == "=" and Orientation.Dual == 0

    for err in 2, "x", None:
        with pytest.raises(ValueError):
            Orientation(err)


# def test_orientation_flip():
#     orientation = Orientation.Forward
#
#     assert orientation.flip() is orientation
#     assert orientation == Orientation.Reverse
#
#     assert orientation.flip() is orientation
#     assert orientation == Orientation.Forward
#
#     orientation = orientation.Dual
#     assert orientation.flip() is orientation
#     assert orientation == Orientation.Dual


def test_orientation_flipped():
    orientation = Orientation.Forward
    assert orientation.flipped() == Orientation.Reverse
    assert orientation == Orientation.Forward
    assert orientation.flipped().flipped() == orientation

    orientation = Orientation.Dual
    assert orientation.flipped() == Orientation.Dual
    assert orientation == Orientation.Dual
    assert orientation.flipped().flipped() == orientation


def test_orientation_order():
    assert Orientation.Forward > Orientation.Dual > Orientation.Reverse
    assert "+" > Orientation.Dual > Orientation.Reverse
    assert Orientation.Forward > "=" > Orientation.Reverse
    assert Orientation.Forward > Orientation.Dual > "-"

    assert Orientation.Reverse < Orientation.Dual < Orientation.Forward
    assert "-" < Orientation.Dual < Orientation.Forward
    assert Orientation.Reverse < "=" < Orientation.Forward
    assert Orientation.Reverse < Orientation.Dual < "+"


def test_orientation_str():
    assert str(Orientation.Forward) == Orientation.Forward.symbol() == "+"
    assert str(Orientation.Reverse) == Orientation.Reverse.symbol() == "-"
    assert str(Orientation.Dual) == Orientation.Dual.symbol() == "="


def test_orientation_repr():
    assert repr(Orientation.Forward) == "Orientation[+]"
    assert repr(Orientation.Reverse) == "Orientation[-]"
    assert repr(Orientation.Dual) == "Orientation[=]"


def test_orientation_pickle():
    for obj in Orientation.Forward, Orientation.Reverse, Orientation.Dual:
        assert pickle.loads(pickle.dumps(obj)) == obj
