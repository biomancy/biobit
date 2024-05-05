from pathlib import Path

import pytest

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
    assert repr(run) == "SeqRun(run1, illumina, paired-end, (file1.fastq, file2.fastq), 1000, None, Description)"
    assert str(run) == ("SeqRun(run1):\n"
                        "\tMachine: illumina\n"
                        "\tLayout: paired-end\n"
                        "\tFiles: file1.fastq, file2.fastq\n"
                        "\tReads: 1000\n"
                        "\tBases: .\n"
                        "\tDescription: Description")


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
