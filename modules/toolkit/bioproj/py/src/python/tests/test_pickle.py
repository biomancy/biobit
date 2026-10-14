import pickle
from pathlib import Path

import pytest
from biobit.toolkit.bioproj import (
    FilePath,
    Layout,
    MatesOrientation,
    Project,
    ProjectId,
    Strandedness,
)
from biobit.toolkit.bioproj.data import Data, asset, dataset
from biobit.toolkit.bioproj.design import (
    GroupContrastId,
    MatchedPairsId,
    UnitContrastId,
    UnitId,
    UnitSetId,
)
from biobit.toolkit.bioproj.provenance import (
    SampleId,
    Source,
    SourceId,
    acquisition,
    library,
)


def test_project_pickle(project: Project) -> None:
    restored = pickle.loads(pickle.dumps(project))
    assert type(restored) is Project
    assert restored == project


def test_domain_and_record_wrappers_pickle(project: Project) -> None:
    domains = [project.provenance, project.data, project.designs]
    indexes = [
        project.provenance.sources,
        project.provenance.samples,
        project.provenance.libraries,
        project.provenance.acquisitions,
        project.data.assets,
        project.data.datasets,
        project.designs.units,
        project.designs.designs,
    ]
    records = [record for index in indexes for record in index.values()]
    for value in [*domains, *indexes, *records]:
        restored = pickle.loads(pickle.dumps(value))
        assert type(restored) is type(value)
        assert restored == value


def test_typed_ids_pickle() -> None:
    for kind in (
        ProjectId,
        SourceId,
        SampleId,
        library.P5P7Id,
        acquisition.IlluminaSingleEndSequencingId,
        acquisition.IlluminaPairedEndSequencingId,
        asset.FastqId,
        dataset.FastqId,
        dataset.PairedFastqId,
        UnitId,
        UnitSetId,
        UnitContrastId,
        GroupContrastId,
        MatchedPairsId,
    ):
        value = kind("ENTITY_1")
        # Protocol 0 also exercises __reduce__, avoiding constructor-only pickle.
        for protocol in (0, pickle.HIGHEST_PROTOCOL):
            restored = pickle.loads(pickle.dumps(value, protocol=protocol))
            assert type(restored) is kind
            assert restored == value
            assert hash(restored) == hash(value)


def test_file_paths_pickle(tmp_path: Path, data: Data) -> None:
    root = tmp_path / "root"
    root.mkdir()
    relative = FilePath.from_relative("reads/../reads-µ.fq.gz", root)
    absolute = FilePath.from_absolute(tmp_path / "reads-µ.fq.gz")
    retained = data.assets["AST_R1"].location
    encoded = [pickle.dumps(value) for value in (relative, absolute, retained)]
    root.rmdir()
    for value, state in zip((relative, absolute, retained), encoded, strict=True):
        restored = pickle.loads(state)
        assert type(restored) is FilePath
        assert restored == value
        assert str(restored) == str(value)
        assert restored.absolute == value.absolute
        assert hash(restored) == hash(value)


def test_pickle_rejects_invalid_bitcode() -> None:
    for value in (
        SourceId("SRC1"),
        Source(SourceId("SRC1")).meta,
        FilePath.from_absolute(Path.cwd() / "reads.fq.gz"),
        Strandedness.Forward,
        MatesOrientation.Inward,
        Layout.Single("Forward"),
        Layout.Paired("Reverse", "Inward"),
    ):
        restore, (state,) = value.__reduce__()
        assert isinstance(state, bytes)
        # Even a zero-payload variant (MatesOrientation.Inward) must reject
        # trailing bytes. Domain validation is intentionally skipped on restore.
        with pytest.raises(ValueError):
            restore(state + b"trailing")
