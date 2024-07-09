import pytest

from biobit_core_py.loc import Strand


def test_strand_new():
    assert Strand.Forward == Strand(1) == Strand("+")
    assert Strand.Reverse == Strand(-1) == Strand("-")

    for err in 0, ".":
        with pytest.raises(ValueError):
            Strand(err)


def test_strand_flip():
    strand = Strand.Forward

    assert strand.flip() is None
    assert strand == Strand.Reverse

    assert strand.flip() is None
    assert strand == Strand.Forward


def test_strand_flipped():
    strand = Strand.Forward
    assert strand.flipped() == Strand.Reverse
    assert strand == Strand.Forward
    assert strand.flipped().flipped() == strand


def test_strand_str():
    assert str(Strand.Forward) == Strand.Forward.symbol() == "+"
    assert str(Strand.Reverse) == Strand.Reverse.symbol() == "-"


def test_strand_repr():
    assert repr(Strand.Forward) == "Strand[+]"
    assert repr(Strand.Reverse) == "Strand[-]"
