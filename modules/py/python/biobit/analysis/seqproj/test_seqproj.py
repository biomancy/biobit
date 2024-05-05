from pathlib import Path

import pytest

from .library import Library, Stranding
from .sample import Sample
from .seqrun import SeqLayout, SeqRun


def test_seqlayout_normalize():
    assert SeqLayout.normalize("paired") == SeqLayout.Paired
    assert SeqLayout.normalize("pe") == SeqLayout.Paired
    assert SeqLayout.normalize(SeqLayout.Paired) == SeqLayout.Paired

    assert SeqLayout.normalize("single") == SeqLayout.Single
    assert SeqLayout.normalize("se") == SeqLayout.Single
    assert SeqLayout.normalize(SeqLayout.Single) == SeqLayout.Single

    with pytest.raises(ValueError):
        SeqLayout.normalize("invalid")


def test_seqlayout_str():
    assert str(SeqLayout.Paired) == "paired-end"
    assert str(SeqLayout.Single) == "single-end"


def test_seqrun():
    run = SeqRun("run1", "illumina", "pe", ("file1.fastq", "file2.fastq"), 1000, None, "Description")
    assert run.ind == "run1"
    assert run.machine == "illumina"
    assert run.layout == SeqLayout.Paired
    assert run.files == (Path("file1.fastq"), Path("file2.fastq"))
    assert run.reads == 1000
    assert run.bases is None
    assert run.description == "Description"
    # assert repr(run) == "SeqRun(run1, illumina, paired-end, (file1.fastq, file2.fastq), 1000, None, Description)"
    # assert str(run) == ("SeqRun(run1):\n"
    #                     "\tMachine: illumina\n"
    #                     "\tLayout: paired-end\n"
    #                     "\tFiles: file1.fastq, file2.fastq\n"
    #                     "\tReads: 1000\n"
    #                     "\tBases: .\n"
    #                     "\tDescription: Description")


def test_seqrun_validators():
    ind, machine, layout, files, reads, bases, description = \
        "run1", "illumina", "pe", ("file1.fastq", "file2.fastq"), 1000, None, "Description"

    with pytest.raises(ValueError):
        SeqRun("", machine, layout, files, reads, bases, description)
    with pytest.raises(ValueError):
        SeqRun(ind, "", layout, files, reads, bases, description)
    with pytest.raises(ValueError):
        SeqRun(ind, machine, "invalid", files, reads, bases, description)
    with pytest.raises(ValueError):
        SeqRun(ind, machine, SeqLayout.Single, [], reads, bases, description)
    with pytest.raises(ValueError):
        SeqRun(ind, machine, SeqLayout.Single, ("f_1.fq", "f_2.fq"), reads, bases, description)
    with pytest.raises(ValueError):
        SeqRun(ind, machine, SeqLayout.Paired, ("f_1.fq", "f_2.fq", "f_3.fq"), reads, bases, description)
    with pytest.raises(ValueError):
        SeqRun(ind, machine, layout, files, 0, bases, description)
    with pytest.raises(ValueError):
        SeqRun(ind, machine, layout, files, None, 0, description)


def test_sample_creation():
    sample = Sample("S1", ("Homo sapiens",), {"Confluence": "75%", "Source": "HeLa"}, "Description")
    assert sample.ind == "S1"
    assert sample.organism == ("Homo sapiens",)
    assert sample.attributes == {"Confluence": "75%", "Source": "HeLa"}
    assert sample.description == "Description"
    # assert repr(sample) == f"Sample(S1, ('Homo sapiens',), {{'Confluence': '75%', 'Source': 'HeLa'}}, Description)"
    # assert str(sample) == (f"Sample(S1):\n"
    #                        f"\tOrganism: Homo sapiens\n"
    #                        f"\tAttributes: Confluence=75%, Source=HeLa\n"
    #                        f"\tDescription: Description")


def test_sample_without_description():
    sample = Sample("Mmus", ("Mus musculus",))
    assert sample.ind == "Mmus"
    assert sample.organism == ("Mus musculus",)
    assert sample.attributes == {}
    assert sample.description is None


def test_sample_with_empty_id():
    with pytest.raises(ValueError):
        Sample("", ("Homo sapiens", "HSV-1"))


def test_sample_with_empty_organism():
    with pytest.raises(ValueError):
        Sample("sample3", ())


def test_sample_with_empty_attributes():
    with pytest.raises(ValueError):
        Sample("Sample", ("Organism",), {"Confluence": ""})


def test_library_creation():
    library = Library(source=("DNA",), selection=("PCR",), stranding=Stranding.Forward)
    assert library.source == ("DNA",)
    assert library.selection == ("PCR",)
    assert library.stranding == Stranding.Forward
    # assert repr(library) == "Library(('DNA',), ('PCR',), forward)"
    # assert str(library) == ("Library:\n"
    #                         "\tSource: DNA\n"
    #                         "\tSelection: PCR\n"
    #                         "\tStranding: forward")


def test_library_stranding_normalization():
    for (strings, expected) in [
        (["f", "forward"], Stranding.Forward),
        (["r", "reverse"], Stranding.Reverse),
        (["x", "unknown"], Stranding.Unknown),
        (["u", "unstranded"], Stranding.Unstranded)
    ]:
        for s in strings:
            assert Stranding.normalize(s) == expected


def test_library_stranding_normalization_invalid():
    for string in "invalid", "":
        with pytest.raises(ValueError):
            Stranding.normalize(string)


def test_library_creation_invalid_source():
    with pytest.raises(ValueError):
        Library(source=(), selection=("PCR",), stranding=Stranding.Forward)


def test_library_creation_invalid_selection():
    with pytest.raises(ValueError):
        Library(source=("DNA",), selection=(), stranding=Stranding.Forward)
