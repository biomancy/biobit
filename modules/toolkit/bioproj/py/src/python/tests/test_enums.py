import pickle
from operator import ge, gt, le, lt

import pytest
from biobit.toolkit.bioproj import MatesOrientation, Strandedness
from biobit.toolkit.bioproj.provenance import library


@pytest.mark.parametrize(
    ("name", "value"),
    [
        ("Forward", Strandedness.Forward),
        ("Reverse", Strandedness.Reverse),
        ("Unstranded", Strandedness.Unstranded),
        ("Inward", MatesOrientation.Inward),
    ],
)
def test_enum_python_protocols(
    name: str, value: Strandedness | MatesOrientation
) -> None:
    kind = type(value)
    assert value == kind(value) == kind(name)  # type: ignore[arg-type]
    assert str(value) == name
    assert repr(value) == f"{kind.__name__}[{name}]"
    assert hash(value) == hash(name)
    assert name not in {value}
    assert value not in {name}
    restored = pickle.loads(pickle.dumps(value))
    assert type(restored) is kind
    assert restored == value


@pytest.mark.parametrize(
    ("kind", "name"),
    [
        (Strandedness, "F"),
        (Strandedness, "invalid"),
        (MatesOrientation, "I"),
        (MatesOrientation, "invalid"),
    ],
)
def test_enum_inputs_require_full_names(
    kind: type[Strandedness] | type[MatesOrientation], name: str
) -> None:
    with pytest.raises(ValueError):
        kind(name)  # type: ignore[arg-type]


@pytest.mark.parametrize("value", [Strandedness.Forward, MatesOrientation.Inward])
def test_enum_comparisons_require_the_same_type(
    value: Strandedness | MatesOrientation,
) -> None:
    for other in str(value), None, 1:
        assert value != other
        assert other != value
        for compare in lt, le, gt, ge:
            with pytest.raises(TypeError):
                compare(value, other)
            with pytest.raises(TypeError):
                compare(other, value)


def test_strandedness_ordering() -> None:
    assert Strandedness.Reverse < Strandedness.Unstranded < Strandedness.Forward


def test_rna_accepts_optional_strandedness_inputs() -> None:
    for value in Strandedness.Forward, Strandedness.Reverse, Strandedness.Unstranded:
        assert library.RNA(strandedness=str(value)).strandedness == value  # type: ignore[arg-type]
        assert library.RNA(strandedness=value).strandedness == value
    unreported = library.RNA()
    assert unreported.strandedness is None
    assert unreported == library.RNA(strandedness=None)
    assert unreported != library.RNA(strandedness="Unstranded")
