from pathlib import Path

import pytest

from . import yaml
from ..experiment import Experiment
from ..library import Library, Stranding
from ..project import Project
from ..sample import Sample
from ..seqrun import SeqLayout, SeqRun


def _ensure_correctness(project: Project, serializer, deserializer):
    serialized = serializer(project)
    deserialized = deserializer(serialized)
    assert project == deserialized


@pytest.mark.parametrize("runs", [
    [SeqRun("RNA-seq", "illumina", SeqLayout.Paired, (Path("file1.fastq"), Path("file2.fastq")), 1000000, 200000000)],
    [
        SeqRun("DNA-seq", "ONT", SeqLayout.Single, (Path("file1.fastq"),)),
        SeqRun("dRNA-seq", "future-ONT", SeqLayout.Paired, (Path("file1.fastq"), Path("file2.fastq")), 1_000_000)
    ]
])
@pytest.mark.parametrize("sample", [
    Sample("S1", ("Homo sapiens", "HSV-1")),
    Sample("S2", ("Mus musculus",), {"Cells": "MEF", "Confluence": "85%"}, "My super experiment")
])
@pytest.mark.parametrize("lib", [
    Library(("transcriptome",), ("poly-A", "nuclear fraction"), Stranding.Unknown),
    Library(("DNA",), ("Total DNA",), Stranding.Reverse, {"Efficiency": "10%"}),
])
def test_yaml_adapter(runs, sample, lib):
    for exp in [
        Experiment("Experiment", sample, lib, runs),
        Experiment("Experiment", sample, lib, runs, {"A": "B"}),
        Experiment("Experiment", sample, lib, runs, {"A": "B", "C": "D"}, "Test"),
    ]:
        project = Project("Project", (exp,), (sample,))

        _ensure_correctness(project, yaml.dumps, yaml.loads)
