import pickle

import pytest
from biobit.toolkit.bioproj import Layout, MatesOrientation, Strandedness


def test_layout_single() -> None:
    layout = Layout.Single(Strandedness.Forward)
    assert isinstance(layout, Layout.Single) and isinstance(layout, Layout)
    assert layout.strandedness == Strandedness.Forward
    assert isinstance(layout.strandedness, Strandedness)
    assert Layout.Single("Forward") == layout
    assert Layout.Single(Strandedness.Reverse) != layout


def test_layout_paired() -> None:
    layout = Layout.Paired(Strandedness.Forward, MatesOrientation.Inward)
    assert isinstance(layout, Layout.Paired) and isinstance(layout, Layout)
    assert layout.strandedness == Strandedness.Forward
    assert layout.orientation == MatesOrientation.Inward
    from_names = Layout.Paired("Forward", "Inward")
    assert isinstance(from_names.strandedness, Strandedness)
    assert isinstance(from_names.orientation, MatesOrientation)
    assert from_names == layout
    assert Layout.Paired(Strandedness.Reverse, MatesOrientation.Inward) != layout


def test_layout_pickle() -> None:
    for layout in [Layout.Single("Forward"), Layout.Paired("Reverse", "Inward")]:
        for protocol in (0, pickle.HIGHEST_PROTOCOL):
            restored = pickle.loads(pickle.dumps(layout, protocol=protocol))
            assert type(restored) is type(layout)
            assert restored == layout
            assert restored.strandedness == layout.strandedness
            if isinstance(layout, Layout.Paired):
                assert restored.orientation == layout.orientation


def test_layout_subclass_contract() -> None:
    with pytest.raises(TypeError):
        Layout()
    for variant in [Layout.Single, Layout.Paired]:
        with pytest.raises(TypeError):
            type("ExtraVariant", (variant,), {})

    single = Layout.Single("Forward")
    paired = Layout.Paired("Forward", "Inward")
    assert not hasattr(single, "orientation")
    assert single != paired
    assert single < paired
    assert hash(single) == hash(Layout.Single("Forward"))
    assert hash(paired) == hash(Layout.Paired("Forward", "Inward"))
    assert len({single, Layout.Single("Forward"), paired}) == 2
    with pytest.raises(AttributeError):
        single.strandedness = Strandedness.Reverse  # type: ignore[misc]


def test_layout_patterns() -> None:
    for layout in [Layout.Single("Reverse"), Layout.Paired("Forward", "Inward")]:
        match layout:
            case Layout.Single(strandedness):
                assert strandedness == Strandedness.Reverse
            case Layout.Paired(strandedness, orientation):
                assert strandedness == Strandedness.Forward
                assert orientation == MatesOrientation.Inward
            case _:
                pytest.fail("layout did not match its variant")
