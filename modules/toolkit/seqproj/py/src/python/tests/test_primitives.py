import pickle
from pathlib import Path

import pytest

from biobit.toolkit.seqproj.layout import Layout, MatesOrientation
from biobit.toolkit.seqproj.library import Library, Strandedness
from biobit.toolkit.seqproj.run import Run
from biobit.toolkit.seqproj.sample import Sample


def test_layout_single():
    layout = Layout.Single("ABC.fastq")
    path = Path("ABC.fastq")

    assert isinstance(layout, Layout.Single) and isinstance(layout, Layout)
    assert layout.file == path

    assert Layout.Single(path) == layout
    assert Layout.Single("DEF.fastq") != layout

    assert pickle.loads(pickle.dumps(layout)) == layout

    with pytest.raises(TypeError):
        Layout.Single(None)


def test_layout_paired():
    layout = Layout.Paired(MatesOrientation.Inward, ("A", "B"))
    files = (Path("A"), Path("B"))

    assert isinstance(layout, Layout.Paired) and isinstance(layout, Layout)
    assert layout.orientation == MatesOrientation.Inward
    assert layout.files == files

    assert Layout.Paired(MatesOrientation.Inward, ("A", "B")) == layout
    assert Layout.Paired(MatesOrientation.Inward, ("A", "C")) != layout

    assert pickle.loads(pickle.dumps(layout)) == layout

    for args in [
        (MatesOrientation.Inward, None),
        ("Invalid", ("A", "B"))
    ]:
        with pytest.raises(TypeError):
            Layout.Paired(*args)


def test_run():
    run = Run("run1", Layout.Single("file1.fastq"), "illumina", 1000, None, "Description")
    assert run.ind == "run1"
    assert run.machine == "illumina"
    assert run.layout == Layout.Single("file1.fastq")
    assert run.reads == 1000
    assert run.bases is None
    assert run.description == "Description"

    assert pickle.loads(pickle.dumps(run)) == run


def test_run_validators():
    pelayout = Layout.Paired(MatesOrientation.Inward, ("file1.fastq", "file2.fastq"))
    ind, machine, reads, bases, description = "run1", "illumina", 123, 1000, "Description"

    Run(ind, pelayout, machine, reads, bases, description)

    for args in [
        ("", pelayout, machine, reads, bases, description),
        (ind, pelayout, "", reads, bases, description),
        # (ind, machine, "invalid", reads, bases, description),
        # (ind, machine, Layout.Single("A"), reads, bases, description),
        (ind, pelayout, machine, 0, bases, description),
        (ind, pelayout, machine, None, 0, description),
    ]:
        with pytest.raises(ValueError):
            Run(*args)


def test_sample_creation():
    sample = Sample("S1", {"Homo sapiens"}, {"Confluence": "75%", "Source": "HeLa"}, "Description")
    assert sample.ind == "S1"
    assert sample.organism == {"Homo sapiens"}
    assert sample.attributes == {"Confluence": "75%", "Source": "HeLa"}
    assert sample.description == "Description"

    assert pickle.loads(pickle.dumps(sample)) == sample


def test_sample_without_description():
    sample = Sample("Mmus", {"Mus musculus"})
    assert sample.ind == "Mmus"
    assert sample.organism == {"Mus musculus"}
    assert sample.attributes == {}
    assert sample.description is None


def test_sample_with_empty_id():
    with pytest.raises(ValueError):
        Sample("", {"Homo sapiens", "HSV-1"})


def test_sample_with_empty_organism():
    with pytest.raises(ValueError):
        Sample("sample3", set())


def test_sample_with_empty_attributes():
    with pytest.raises(ValueError):
        Sample("Sample", {"Organism"}, {"Confluence": ""})


def test_library_creation():
    library = Library(source={"DNA", }, selection={"PCR", }, strandedness=Strandedness.Unstranded)
    assert library.source == {"DNA"}
    assert library.selection == {"PCR"}
    assert library.strandedness == Strandedness.Unstranded
    assert library.attributes == {}

    assert pickle.loads(pickle.dumps(library)) == library

    for kwargs in [
        dict(source=set(), selection={"PCR"}, strandedness=Strandedness.Forward),
        dict(source={"DNA"}, selection=set(), strandedness=Strandedness.Reverse)
    ]:
        with pytest.raises(ValueError):
            Library(**kwargs)
