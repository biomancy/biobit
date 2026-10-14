from pathlib import Path

import pytest
from biobit.toolkit.bioproj import Project
from biobit.toolkit.bioproj.io import json

# Resolve the umbrella package's symlink before locating the module resources.
RESOURCES = Path(__file__).resolve().parents[4] / "resources"


def valid_fixtures() -> list[Path]:
    directory = RESOURCES / "valid"
    paths = sorted(
        path
        for path in directory.iterdir()
        if path.suffix == ".json" and not path.name.endswith(".error.json")
    )
    assert paths, f"No JSON fixtures in {directory}"
    return paths


@pytest.mark.parametrize("path", valid_fixtures(), ids=lambda path: path.stem)
def test_valid_json_corpus_loads(path: Path) -> None:
    assert isinstance(json.read(path), Project)


@pytest.mark.parametrize(
    ("name", "exception"),
    [
        ("missing_source", KeyError),
        ("blank_design_name", ValueError),
        ("unsupported_major_schema", ValueError),
    ],
)
def test_invalid_json_corpus_raises(name: str, exception: type[Exception]) -> None:
    with pytest.raises(exception):
        json.read(RESOURCES / "invalid" / f"{name}.json")


def test_file_round_trip_and_write_options(project: Project, tmp_path: Path) -> None:
    original = project
    path = tmp_path / "bioproj.json"
    json.write(path, original)
    loaded = json.read(path)
    assert loaded == original
    assert list(tmp_path.iterdir()) == [path]

    json.write(path, loaded, pretty=False)
    compact = path.read_bytes()
    assert compact.endswith(b"\n")
    assert b"\n  " not in compact
    assert json.read(path) == original


def test_relative_document_path(
    project: Project, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.chdir(tmp_path)
    json.write("bioproj.json", project)
    assert json.read("bioproj.json") == project


def test_json_errors_use_standard_exceptions_with_messages(tmp_path: Path) -> None:
    with pytest.raises(OSError, match="failed to open project document.*missing.json"):
        json.read(tmp_path / "missing.json")

    malformed = tmp_path / "malformed.json"
    malformed.write_bytes(b"not-json")
    with pytest.raises(ValueError, match="failed to parse project JSON"):
        json.read(malformed)
