import pytest

from biobit_core_py.loc import Orientation


def test_orientation_new():
    assert Orientation(1) == Orientation("+") == Orientation.Forward
    assert Orientation(-1) == Orientation("-") == Orientation.Reverse
    assert Orientation(0) == Orientation("=") == Orientation.Dual

    for err in 2, "x", None:
        with pytest.raises(ValueError):
            Orientation(err)


def test_orientation_flip():
    orientation = Orientation.Forward

    assert orientation.flip() is None
    assert orientation == Orientation.Reverse

    assert orientation.flip() is None
    assert orientation == Orientation.Forward

    orientation = orientation.Dual
    assert orientation.flip() is None
    assert orientation == Orientation.Dual


def test_orientation_flipped():
    orientation = Orientation.Forward
    assert orientation.flipped() == Orientation.Reverse
    assert orientation == Orientation.Forward
    assert orientation.flipped().flipped() == orientation

    orientation = Orientation.Dual
    assert orientation.flipped() == Orientation.Dual
    assert orientation == Orientation.Dual
    assert orientation.flipped().flipped() == orientation


def test_orientation_str():
    assert str(Orientation.Forward) == Orientation.Forward.symbol() == "+"
    assert str(Orientation.Reverse) == Orientation.Reverse.symbol() == "-"
    assert str(Orientation.Dual) == Orientation.Dual.symbol() == "="


def test_orientation_repr():
    assert repr(Orientation.Forward) == "Orientation[+]"
    assert repr(Orientation.Reverse) == "Orientation[-]"
    assert repr(Orientation.Dual) == "Orientation[=]"
