import pytest

from biobit.core import strdeductor, Strand


def test_get():
    assert strdeductor.get("f/s") == strdeductor._fs
    assert strdeductor.get("s/f") == strdeductor._sf
    assert strdeductor.get("f") == strdeductor._f
    assert strdeductor.get("s") == strdeductor._s
    with pytest.raises(ValueError):
        strdeductor.get("invalid")  # type: ignore


def test_f():
    assert strdeductor._f(_AlignedSegmentStub(True)) == Strand.fwd
    assert strdeductor._f(_AlignedSegmentStub(False)) == Strand.rev


def test_s():
    assert strdeductor._s(_AlignedSegmentStub(True)) == Strand.rev
    assert strdeductor._s(_AlignedSegmentStub(False)) == Strand.fwd


def test_fs():
    assert strdeductor._fs(_AlignedSegmentStub(True, True)) == Strand.fwd
    assert strdeductor._fs(_AlignedSegmentStub(True, False)) == Strand.rev
    assert strdeductor._fs(_AlignedSegmentStub(False, True)) == Strand.rev
    assert strdeductor._fs(_AlignedSegmentStub(False, False)) == Strand.fwd


def test_sf():
    assert strdeductor._sf(_AlignedSegmentStub(True, True)) == Strand.rev
    assert strdeductor._sf(_AlignedSegmentStub(True, False)) == Strand.fwd
    assert strdeductor._sf(_AlignedSegmentStub(False, True)) == Strand.fwd
    assert strdeductor._sf(_AlignedSegmentStub(False, False)) == Strand.rev


class _AlignedSegmentStub:
    def __init__(self, is_reverse, is_read1=None):
        self.is_reverse = is_reverse
        self.is_read1 = is_read1
