import pytest

from . import yaml
from ..experiment import Experiment
from ..layout import Layout, MatesOrientation
from ..library import Library, Strandedness
from ..project import Project
from ..run import Run
from ..sample import Sample


def _ensure_correctness(project: Project, serializer, deserializer):
    serialized = serializer(project)
    deserialized = deserializer(serialized)
    assert project == deserialized


@pytest.mark.parametrize("runs", [
    [
        Run("RNA-seq", Layout.Paired(MatesOrientation.Inward, ("file1.fastq", "file2.fastq")), "illumina", 1)
    ],
    [
        Run("DNA-seq", Layout.Single("F1.fastq"), description="Description"),
        Run("RNA-seq", Layout.Paired(MatesOrientation.Inward, ("file1.fastq", "file2.fastq")), "illumina", 19, 15)
    ],
    [
        Run("DNA-seq", Layout.Single("file1.fastq")),
        Run("dRNA-seq", Layout.Paired(MatesOrientation.Inward, ("file1.fastq", "file2.fastq")), "future-ONT", 3)
    ]
])
@pytest.mark.parametrize("sample", [
    Sample("S1", {"Homo sapiens", "HSV-1"}),
    Sample("S2", {"Mus musculus"}, {"Cells": "MEF", "Confluence": "85%"}, "My super experiment")
])
@pytest.mark.parametrize("lib", [
    Library({"transcriptome", }, {"poly-A", "nuclear fraction"}, Strandedness.Reverse),
    Library({"DNA"}, {"Total DNA"}, None, {"Efficiency": "10%"}),
])
def test_yaml_adapter(runs, sample, lib):
    for exp in [
        Experiment("Experiment", sample, lib, runs),
        Experiment("Experiment", sample, lib, runs, {"A": "B"}),
        Experiment("Experiment", sample, lib, runs, {"A": "B", "C": "D"}, "Test"),
    ]:
        project = Project("Project", (exp,), (sample,))

        _ensure_correctness(project, yaml.dumps, yaml.loads)
