import pickle

import pytest

from biobit.core.ngs import Strandedness, MatesOrientation, Layout


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


def test_layout_pickle():
    for strandedness in Strandedness.Forward, Strandedness.Reverse, Strandedness.Unstranded:
        layout = Layout.Single(strandedness)
        assert layout == pickle.loads(pickle.dumps(layout))

        for orientation in MatesOrientation.Inward,:
            layout = Layout.Paired(strandedness, orientation)
            assert layout == pickle.loads(pickle.dumps(layout))
