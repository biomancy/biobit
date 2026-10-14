from pathlib import Path

import pytest
from biobit.toolkit.bioproj import FilePath, Project, ProjectId, Strandedness
from biobit.toolkit.bioproj.data import Data, asset, dataset
from biobit.toolkit.bioproj.design import (
    Designs,
    GroupContrast,
    GroupContrastId,
    MatchedPairs,
    MatchedPairsId,
    Unit,
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


@pytest.fixture
def provenance() -> Provenance:
    source = Source(
        SourceId("SRC1"),
        organism="Homo sapiens",
        meta={"control": True, "strain": "reference strain"},
    )
    sample = Sample(SampleId("SMP1"), sources=[source.id])
    prepared = library.P5P7(
        library.P5P7Id("LIB1"),
        samples=[sample.id],
        input=library.RNA(strandedness=Strandedness.Forward),
    )
    paired = acquisition.IlluminaPairedEndSequencing(
        acquisition.IlluminaPairedEndSequencingId("ACQ1"), libraries=[prepared.id]
    )
    single = acquisition.IlluminaSingleEndSequencing(
        acquisition.IlluminaSingleEndSequencingId("ACQ_SINGLE"), libraries=[prepared.id]
    )
    return Provenance(
        sources=[source],
        samples=[sample],
        libraries=[prepared],
        acquisitions=[paired, single],
    )


@pytest.fixture
def data(tmp_path: Path) -> Data:
    read1 = asset.Fastq(
        asset.FastqId("AST_R1"),
        location=FilePath.from_relative("reads_R1.fq.gz", tmp_path),
    )
    read2 = asset.Fastq(
        asset.FastqId("AST_R2"),
        location=FilePath.from_relative("reads_R2.fq.gz", tmp_path),
    )
    single_read = asset.Fastq(
        asset.FastqId("AST_SINGLE"),
        location=FilePath.from_relative("single.fq.gz", tmp_path),
    )
    paired_data = dataset.PairedFastq(
        dataset.PairedFastqId("DATA1"),
        acquisition=acquisition.IlluminaPairedEndSequencingId("ACQ1"),
        pairs=[(read1.id, read2.id)],
    )
    single_data = dataset.Fastq(
        dataset.FastqId("DATA_SINGLE"),
        acquisition=acquisition.IlluminaSingleEndSequencingId("ACQ_SINGLE"),
        assets=[single_read.id],
    )
    return Data(
        assets=[read1, read2, single_read],
        datasets=[paired_data, single_data],
    )


@pytest.fixture
def designs() -> Designs:
    paired_unit = Unit(
        UnitId("UNIT1"),
        acquisitions=[acquisition.IlluminaPairedEndSequencingId("ACQ1")],
    )
    single_unit = Unit(
        UnitId("UNIT_SINGLE"),
        acquisitions=[acquisition.IlluminaSingleEndSequencingId("ACQ_SINGLE")],
    )
    joint = UnitSet(
        UnitSetId("DES1"), name="Joint analysis", units=[paired_unit.id, single_unit.id]
    )
    contrast = UnitContrast(
        UnitContrastId("DES_CONTRAST"),
        name="Control vs treatment",
        control=single_unit.id,
        treatment=paired_unit.id,
    )
    grouped = GroupContrast(
        GroupContrastId("DES_GROUP"),
        name="Grouped comparison",
        control=[single_unit.id],
        treatment=[paired_unit.id],
    )
    matched = MatchedPairs(
        MatchedPairsId("DES_MATCHED"),
        name="Matched measurements",
        pairs=[(single_unit.id, paired_unit.id)],
    )
    return Designs(
        units=[paired_unit, single_unit],
        designs=[joint, contrast, grouped, matched],
    )


@pytest.fixture
def project(provenance: Provenance, data: Data, designs: Designs) -> Project:
    return Project(
        ProjectId("PROJ1"),
        provenance=provenance,
        data=data,
        designs=designs,
        description="Example released project",
    )
