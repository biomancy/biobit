import pytest

from biobit.core import Orientation


def test_orientation_str_repr():
    assert str(Orientation.fwd) == "+"
    assert repr(Orientation.fwd) == "+"
    assert str(Orientation.rev) == "-"
    assert repr(Orientation.rev) == "-"
    assert str(Orientation.dual) == "."
    assert repr(Orientation.dual) == "."


def test_orientation_normalize():
    assert Orientation.normalize("+") == Orientation.fwd
    assert Orientation.normalize(1) == Orientation.fwd
    assert Orientation.normalize(Orientation.fwd) == Orientation.fwd
    assert Orientation.normalize("-") == Orientation.rev
    assert Orientation.normalize(-1) == Orientation.rev
    assert Orientation.normalize(Orientation.rev) == Orientation.rev
    assert Orientation.normalize(".") == Orientation.dual
    assert Orientation.normalize(0) == Orientation.dual
    assert Orientation.normalize(Orientation.dual) == Orientation.dual


def test_orientation_normalize_invalid():
    with pytest.raises(ValueError):
        Orientation.normalize("invalid")
