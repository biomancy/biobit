from pathlib import Path

from . import yaml
from ..experiment import Experiment
from ..library import Library, Stranding
from ..project import Project
from ..sample import Sample
from ..seqrun import SeqLayout, SeqRun


# TODO: Fix the test suite
# - Cover all corner cases
# - Reduce repetitiveness
# - Make the tests scalable for further adapters


def _ensure_correctness(project: Project, serializer, deserializer):
    serialized = serializer(project)
    deserialized = deserializer(serialized)
    assert project == deserialized


def test_yaml_adapter():
    run = SeqRun("my-run", "illumina", SeqLayout.Paired, (Path("file1.fastq"), Path("file2.fastq")), 1000000, 200000000)
    sample = Sample("smpl-ind", ("homo sapiens",), {"tissue": "brain", "cells": "neurons"}, "My best sample ever")
    lib = Library(("transcriptome",), ("poly-A",), Stranding.Forward)
    exp = Experiment("exp-1", sample, lib, (run,))
    project = Project("super-project-ind", (exp,), (sample,))

    _ensure_correctness(project, yaml.dumps, yaml.loads)


def test_yaml_adapter_single_layout():
    run = SeqRun("my-run", "illumina", SeqLayout.Single, (Path("file1.fastq"),), 1000000, 200000000)
    sample = Sample("smpl-ind", ("homo sapiens",), {"tissue": "brain", "cells": "neurons"}, "My best sample ever")
    lib = Library(("transcriptome",), ("poly-A",), Stranding.Forward)
    exp = Experiment("exp-1", sample, lib, (run,))
    project = Project("super-project-ind", (exp,), (sample,))

    _ensure_correctness(project, yaml.dumps, yaml.loads)


def test_yaml_adapter_unknown_stranding():
    run = SeqRun("my-run", "illumina", SeqLayout.Paired, (Path("file1.fastq"), Path("file2.fastq")), 1000000, 200000000)
    sample = Sample("smpl-ind", ("homo sapiens",), {"tissue": "brain", "cells": "neurons"}, "My best sample ever")
    lib = Library(("transcriptome",), ("poly-A",), Stranding.Unknown)
    exp = Experiment("exp-1", sample, lib, (run,))
    project = Project("super-project-ind", (exp,), (sample,))

    _ensure_correctness(project, yaml.dumps, yaml.loads)


def test_yaml_adapter_no_tags():
    run = SeqRun("my-run", "illumina", SeqLayout.Paired, (Path("file1.fastq"), Path("file2.fastq")), 1000000, 200000000)
    sample = Sample("smpl-ind", ("homo sapiens",))
    lib = Library(("transcriptome",), ("poly-A",), Stranding.Forward)
    exp = Experiment("exp-1", sample, lib, (run,))
    project = Project("super-project-ind", (exp,), (sample,))

    _ensure_correctness(project, yaml.dumps, yaml.loads)
