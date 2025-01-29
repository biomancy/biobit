import pickle

import pytest

from biobit.core.loc import Strand


def test_strand_new():
    assert Strand.Forward == Strand(1) == Strand("+") == "+"
    assert Strand.Reverse == Strand(-1) == Strand("-") == "-"

    for err in 0, ".":
        with pytest.raises(ValueError):
            Strand(err)


# def test_strand_flip():
#     strand = Strand.Forward
#
#     assert strand.flip() is strand
#     assert strand == Strand.Reverse
#
#     assert strand.flip() is strand
#     assert strand == Strand.Forward


def test_strand_flipped():
    strand = Strand.Forward
    assert strand.flipped() == Strand.Reverse
    assert strand == Strand.Forward
    assert strand.flipped().flipped() == strand


def test_strand_str():
    assert str(Strand.Forward) == Strand.Forward.symbol() == "+"
    assert str(Strand.Reverse) == Strand.Reverse.symbol() == "-"


def test_strand_order():
    assert Strand.Forward > Strand.Reverse
    assert Strand.Forward > '-'
    assert Strand.Forward > -1

    assert Strand.Reverse < Strand.Forward
    assert Strand.Reverse < '+'
    assert Strand.Reverse < 1


def test_strand_repr():
    assert repr(Strand.Forward) == "Strand[+]"
    assert repr(Strand.Reverse) == "Strand[-]"


def test_strand_pickle():
    for strand in Strand.Forward, Strand.Reverse:
        assert pickle.loads(pickle.dumps(strand)) == strand
