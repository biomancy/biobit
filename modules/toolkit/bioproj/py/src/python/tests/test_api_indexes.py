from collections.abc import ItemsView, Iterator, KeysView, Mapping, ValuesView
from operator import length_hint
from typing import Any

import pytest
from biobit.toolkit.bioproj import Project, ProjectId
from biobit.toolkit.bioproj.provenance import Provenance, Source, SourceId

FAMILIES = [
    ("provenance", "sources"),
    ("provenance", "samples"),
    ("provenance", "libraries"),
    ("provenance", "acquisitions"),
    ("data", "assets"),
    ("data", "datasets"),
    ("designs", "units"),
    ("designs", "designs"),
]


def index(project: Project, domain: str, family: str) -> Mapping[Any, Any]:
    return getattr(getattr(project, domain), family)


@pytest.mark.parametrize(("domain", "family"), FAMILIES)
def test_each_family_index_accepts_typed_and_string_keys(
    project: Project, domain: str, family: str
) -> None:
    records = index(project, domain, family)
    assert records
    for key in records:
        assert records[key] == records[str(key)]


def test_indexes_expose_mapping_views(project: Project) -> None:
    records = project.provenance.acquisitions
    keys, values, items = records.keys(), records.values(), records.items()
    assert isinstance(keys, KeysView)
    assert isinstance(values, ValuesView)
    assert isinstance(items, ItemsView)
    assert list(keys) == list(records)
    assert list(items) == [(key, records[key]) for key in records]
    assert list(values) == [value for _, value in items]
    assert records == dict(items)
    assert dict(items) == records
    assert records != {}

    expected = list(items)
    del records
    assert list(items) == expected


def test_index_iterators_are_independent_and_keep_their_owner() -> None:
    domain = Provenance(
        sources=[Source(SourceId("A")), Source(SourceId("B"))],
        samples=[],
        libraries=[],
        acquisitions=[],
    )
    records = domain.sources
    expected = list(records)
    first, second = iter(records), iter(records)
    assert isinstance(first, Iterator)
    assert iter(first) is first
    assert length_hint(first) == len(expected)
    assert next(first) == expected[0]
    assert length_hint(first) == len(expected) - 1

    del domain, records
    assert list(second) == expected
    assert list(first) == expected[1:]
    assert length_hint(first) == 0


def test_empty_index_views() -> None:
    records = Provenance(sources=[], samples=[], libraries=[], acquisitions=[]).sources
    for view in [records, records.keys(), records.values(), records.items()]:
        assert len(view) == 0
        assert list(view) == []


def test_lookup_handles_python_keys_and_defaults(project: Project) -> None:
    records = project.provenance.sources
    key = next(iter(records))
    fallback = object()
    assert records.get(key=key, default=fallback) == records[key]
    assert records.get(key=str(key)) == records[key]

    mapping: Mapping[Any, Any] = records
    for missing in [type(key)("MISSING"), "bad id", "\ud800"]:
        assert mapping.get(missing, fallback) is fallback
        assert missing not in mapping
        with pytest.raises(KeyError) as caught:
            mapping[missing]
        assert caught.value.args[0] is missing

    for unsupported in [None, ProjectId("SRC1")]:
        assert unsupported not in mapping
        with pytest.raises(TypeError):
            mapping[unsupported]


def test_family_lookup_reports_wrong_variant(project: Project) -> None:
    records = project.data.datasets
    keys = list(records)
    key = keys[0]
    other_kind = next(other for other in keys if type(other) is not type(key))
    wrong_kind = type(other_kind)(str(key))
    assert wrong_kind not in records
    with pytest.raises(TypeError, match="expects .*but identifies"):
        records[wrong_kind]
