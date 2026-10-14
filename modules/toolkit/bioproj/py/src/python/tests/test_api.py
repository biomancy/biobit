import os
from collections.abc import Iterator, Mapping
from pathlib import Path
from typing import Any

import pytest
from biobit.toolkit.bioproj import (
    FilePath,
    Project,
    ProjectId,
    Strandedness,
)
from biobit.toolkit.bioproj.data import Data, asset, dataset
from biobit.toolkit.bioproj.design import (
    Designs,
    GroupContrast,
    GroupContrastId,
    MatchedPairs,
    MatchedPairsId,
    UnitContrast,
    UnitContrastId,
    UnitId,
    UnitSet,
    UnitSetId,
)
from biobit.toolkit.bioproj.provenance import (
    Provenance,
    Sample,
    SampleId,
    Source,
    SourceId,
    acquisition,
    library,
)


def test_file_path_uses_python_path_protocol(tmp_path: Path) -> None:
    root = Path.cwd()
    relative = FilePath.from_relative("reads.fq.gz", root)
    assert str(relative) == "reads.fq.gz"
    assert relative.absolute == root / "reads.fq.gz"
    assert os.fspath(relative) == str(root / "reads.fq.gz")
    assert Path(relative) == root / "reads.fq.gz"

    absolute = root / "reads.fq.gz"
    assert asset.Fastq(asset.FastqId("READS"), location=str(absolute)).location == (
        FilePath.from_absolute(absolute)
    )

    absolute = tmp_path / "reads-µ.fq.gz"
    path = FilePath.from_absolute(absolute)
    assert os.fsencode(os.fspath(path)) == os.fsencode(absolute)


def test_input_conversion_preserves_python_exceptions() -> None:
    class BrokenSequence(list[SourceId]):
        def __iter__(self) -> Iterator[SourceId]:
            raise RuntimeError("iterator failed")

    def broken_sources() -> Iterator[SourceId]:
        yield SourceId("SRC")
        raise RuntimeError("iteration failed")

    class BrokenPath:
        def __fspath__(self) -> str:
            raise RuntimeError("path failed")

    with pytest.raises(RuntimeError, match="iterator failed"):
        Sample(SampleId("SMP"), sources=BrokenSequence())
    with pytest.raises(RuntimeError, match="iteration failed"):
        Sample(SampleId("SMP"), sources=broken_sources())
    with pytest.raises(RuntimeError, match="path failed"):
        asset.Fastq(asset.FastqId("READS"), location=BrokenPath())


def test_collection_constructors_accept_iterables(
    provenance: Provenance, data: Data, designs: Designs
) -> None:
    assert (
        Provenance(
            sources=(source for source in provenance.sources.values()),
            samples=provenance.samples.values(),
            libraries=iter(provenance.libraries.values()),
            acquisitions=provenance.acquisitions.values(),
        )
        == provenance
    )
    assert (
        Data(
            assets=data.assets.values(),
            datasets=(record for record in data.datasets.values()),
        )
        == data
    )
    assert (
        Designs(units=iter(designs.units.values()), designs=designs.designs.values())
        == designs
    )


def test_record_constructors_accept_their_own_collection_getters(
    provenance: Provenance, data: Data, designs: Designs
) -> None:
    sample = provenance.samples["SMP1"]
    assert Sample(sample.id, sources=sample.sources) == sample
    prepared = provenance.libraries["LIB1"]
    assert (
        library.P5P7(prepared.id, samples=prepared.samples, input=prepared.input)
        == prepared
    )
    acquired = provenance.acquisitions["ACQ1"]
    assert (
        acquisition.IlluminaPairedEndSequencing(
            acquired.id, libraries=acquired.libraries
        )
        == acquired
    )
    paired = data.datasets["DATA1"]
    assert (
        dataset.PairedFastq(
            paired.id, acquisition=paired.acquisition, pairs=paired.pairs
        )
        == paired
    )
    matched = designs.designs[MatchedPairsId("DES_MATCHED")]
    assert (
        MatchedPairs(matched.id, name=matched.name, pairs=iter(matched.pairs))
        == matched
    )


def test_project_exposes_its_domains(
    project: Project, provenance: Provenance, data: Data, designs: Designs
) -> None:
    assert project.id == ProjectId("PROJ1")
    assert project.description == "Example released project"
    assert project.provenance == provenance
    assert project.data == data
    assert project.designs == designs


def test_provenance_getters_expose_typed_records(provenance: Provenance) -> None:
    source = provenance.sources["SRC1"]
    assert isinstance(source, Source)
    assert source.organism == "Homo sapiens"
    assert source.meta == {"control": True, "strain": "reference strain"}
    with pytest.raises(AttributeError):
        source.organism = "changed"  # type: ignore[misc]

    sample = provenance.samples["SMP1"]
    assert sample.sources == {SourceId("SRC1")}
    prepared = provenance.libraries["LIB1"]
    assert prepared.samples == {SampleId("SMP1")}
    assert isinstance(prepared.input, library.RNA)
    assert prepared.input.strandedness == Strandedness.Forward
    single = provenance.acquisitions["ACQ_SINGLE"]
    paired = provenance.acquisitions["ACQ1"]
    assert isinstance(single, acquisition.IlluminaSingleEndSequencing)
    assert isinstance(paired, acquisition.IlluminaPairedEndSequencing)
    assert paired.libraries == {library.P5P7Id("LIB1")}


def test_data_getters_expose_dataset_layout(data: Data) -> None:
    single = data.datasets["DATA_SINGLE"]
    assert isinstance(single, dataset.Fastq)
    assert single.acquisition == acquisition.IlluminaSingleEndSequencingId("ACQ_SINGLE")
    assert single.assets == {asset.FastqId("AST_SINGLE")}

    paired = data.datasets["DATA1"]
    assert isinstance(paired, dataset.PairedFastq)
    assert paired.acquisition == acquisition.IlluminaPairedEndSequencingId("ACQ1")
    assert paired.pairs == {(asset.FastqId("AST_R1"), asset.FastqId("AST_R2"))}


def test_design_getters_expose_each_layout(designs: Designs) -> None:
    for key, kind in [
        ("DES1", UnitSet),
        ("DES_CONTRAST", UnitContrast),
        ("DES_GROUP", GroupContrast),
        ("DES_MATCHED", MatchedPairs),
    ]:
        assert isinstance(designs.designs[key], kind)

    assert designs.units["UNIT1"].acquisitions == {
        acquisition.IlluminaPairedEndSequencingId("ACQ1")
    }
    assert designs.designs[UnitSetId("DES1")].units == {
        UnitId("UNIT1"),
        UnitId("UNIT_SINGLE"),
    }
    contrast = designs.designs[UnitContrastId("DES_CONTRAST")]
    assert contrast.control == UnitId("UNIT_SINGLE")
    assert contrast.treatment == UnitId("UNIT1")
    grouped = designs.designs[GroupContrastId("DES_GROUP")]
    assert grouped.control == {UnitId("UNIT_SINGLE")}
    assert grouped.treatment == {UnitId("UNIT1")}
    assert designs.designs[MatchedPairsId("DES_MATCHED")].pairs == {
        (UnitId("UNIT_SINGLE"), UnitId("UNIT1"))
    }


def test_metadata_is_read_only_and_retains_its_owner() -> None:
    # The temporary domain and Source disappear; the metadata view must keep
    # the Rust-owned chain alive on its own.
    metadata = (
        Provenance(
            sources=[Source(SourceId("SRC"), meta={"flag": True})],
            samples=[],
            libraries=[],
            acquisitions=[],
        )
        .sources["SRC"]
        .meta
    )
    assert isinstance(metadata, Mapping)
    assert dict(metadata) == {"flag": True}

    assert metadata == {"flag": True}
    assert {"flag": True} == metadata
    assert Source(SourceId("COPY"), meta=metadata).meta == metadata
    with pytest.raises(TypeError):
        metadata["flag"] = False  # type: ignore[index]


def test_metadata_surrogate_keys_are_missing() -> None:
    metadata = Source(SourceId("SRC"), meta={"flag": True}).meta
    key = "\ud800"
    assert key not in metadata
    assert metadata.get(key, False) is False
    with pytest.raises(KeyError) as caught:
        metadata[key]
    assert caught.value.args == (key,)


@pytest.mark.parametrize("entry", [("flag",), ("flag", True, "extra")])
def test_metadata_rejects_malformed_mapping_entries(entry: tuple[object, ...]) -> None:
    class MalformedMapping(dict[str, bool]):
        # Deliberately violate the items() contract to check boundary validation.
        def items(self) -> Any:
            return [entry]

    with pytest.raises(
        ValueError, match="metadata entries must contain exactly two elements"
    ):
        Source(SourceId("SRC"), meta=MalformedMapping())


def test_typed_inputs_reject_incompatible_python_values() -> None:
    with pytest.raises(TypeError):
        Sample(SampleId("SMP"), sources=["SRC"])  # type: ignore[list-item]
    with pytest.raises(TypeError):
        Source(SourceId("SRC"), meta={"number": 1})  # type: ignore[dict-item]
    with pytest.raises(TypeError):
        Source("SRC")  # type: ignore[arg-type]


def test_id_wrappers_keep_their_type() -> None:
    source_id = SourceId("SAME")
    assert str(source_id) == "SAME"
    assert not hasattr(source_id, "value")
    assert source_id != ProjectId("SAME")
    assert source_id != "SAME"
    with pytest.raises(TypeError):
        _ = SourceId("A") < ProjectId("A")  # type: ignore[operator]


def test_pair_input_requires_two_typed_asset_ids(data: Data) -> None:
    paired = data.datasets[dataset.PairedFastqId("DATA1")]
    with pytest.raises(ValueError, match="tuple of length 2"):
        dataset.PairedFastq(
            dataset.PairedFastqId("BAD"),
            acquisition=paired.acquisition,
            pairs=[(asset.FastqId("AST_R1"),)],  # type: ignore[list-item]
        )


def test_design_names_are_exposed_and_read_only() -> None:
    with pytest.raises(TypeError):
        UnitSet(UnitSetId("DES"), units=[UnitId("UNIT")])  # type: ignore[call-arg]

    design = UnitSet(UnitSetId("DES"), name="Joint analysis", units=[UnitId("UNIT")])
    assert design.name == "Joint analysis"
    with pytest.raises(AttributeError):
        design.name = "Other"  # type: ignore[misc]
