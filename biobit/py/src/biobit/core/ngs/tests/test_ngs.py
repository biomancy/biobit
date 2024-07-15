import pytest

from biobit.core.ngs import Strandedness, MatesOrientation, Layout


def test_strandedness():
    assert Strandedness.Forward == Strandedness(Strandedness.Forward) == Strandedness("F")
    assert Strandedness.Reverse == Strandedness(Strandedness.Reverse) == Strandedness("R")
    assert Strandedness.Unstranded == Strandedness(Strandedness.Unstranded) == Strandedness("U")

    with pytest.raises(ValueError):
        Strandedness("invalid")


def test_mates_orientation():
    assert MatesOrientation.Inward == MatesOrientation(MatesOrientation.Inward) == MatesOrientation("I")

    with pytest.raises(ValueError):
        MatesOrientation("invalid")


def test_layout_single():
    layout = Layout.Single(Strandedness.Forward)
    assert isinstance(layout, Layout.Single) and isinstance(layout, Layout)
    assert layout.strandedness == Strandedness.Forward
    assert Layout.Single(Strandedness.Forward) == layout
    assert Layout.Single(Strandedness.Reverse) != layout
    assert Layout.Single(Strandedness.Unstranded) != layout

    with pytest.raises(TypeError):
        Layout.Single("Invalid")


def test_layout_paired():
    layout = Layout.Paired(Strandedness.Forward, MatesOrientation.Inward)
    assert isinstance(layout, Layout.Paired) and isinstance(layout, Layout)
    assert layout.strandedness == Strandedness.Forward
    assert layout.orientation == MatesOrientation.Inward
    assert Layout.Paired(Strandedness.Forward, MatesOrientation.Inward) == layout
    assert Layout.Paired(Strandedness.Reverse, MatesOrientation.Inward) != layout
    assert Layout.Paired(Strandedness.Unstranded, MatesOrientation.Inward) != layout

    with pytest.raises(TypeError):
        Layout.Paired("Invalid", MatesOrientation.Inward)

    with pytest.raises(TypeError):
        Layout.Paired(Strandedness.Forward, "Invalid")
