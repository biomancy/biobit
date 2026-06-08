import os
import pickle
from pathlib import Path

import pytest

from biobit.core.loc import Interval, Orientation
from biobit.core.ngs import Layout, Strandedness
from biobit.io.fasta import IndexedSources
from biobit.toolkit import reat

RESOURCES = Path(os.environ["BIOBIT_RESOURCES"])
BAM = RESOURCES / "bam" / "RNA-seq.CHM13v2.21-22.bam"
REFERENCE = RESOURCES / "fasta" / "CHM13v2.M-21-22.fa.bgz"


def test_task_api():
    tasks = reat.Task.from_intervals(
        [("chr2", (10, 15)), ("chr1", Interval(20, 30)), ("chr1", (25, 40))],
        100,
    )

    assert [task.seqid for task in tasks] == ["chr1", "chr2"]
    assert tasks[0].envelope == Interval(20, 40)
    assert tasks[0].intervals == [Interval(20, 40)]
    assert pickle.loads(pickle.dumps(tasks)) == tasks


def test_pileup_api():
    counts = reat.Pileup([1, 2], [0, 1], [0, 0], [2, 0], [0, 0], [0, 1])
    assert counts.len() == 2
    assert counts.coverage() == [3, 4]
    assert pickle.loads(pickle.dumps(counts)) == counts

    pileup = reat.SparsePileup([20, 22], counts)
    assert pileup.interval == Interval(20, 23)
    assert pileup.positions() == [20, 22]
    assert pileup.counts() == counts

    assert pickle.loads(pickle.dumps(pileup)) == pileup

    task_pileup = reat.TaskPileup(pileup, ["A", "C"])
    assert task_pileup.pileup() == pileup
    assert task_pileup.reference() == ["A", "C"]
    assert task_pileup.interval == Interval(20, 23)
    assert pickle.loads(pickle.dumps(task_pileup)) == task_pileup

    large = reat.SparsePileup([2**63], reat.Pileup.zeros(1))
    with pytest.raises(ValueError):
        _ = large.interval


def test_selection_api():
    default = reat.selection.Mismatches()
    assert default.minmismatches == 1
    assert default.minfreq == pytest.approx(0.0)

    mismatches = reat.selection.Mismatches(minmismatches=2, minfreq=0.25)
    assert mismatches.minmismatches == 2
    assert mismatches.minfreq == pytest.approx(0.25)
    assert pickle.loads(pickle.dumps(mismatches)) == mismatches

    required = reat.selection.RequiredSites([("chr21", "+", [(100, 101), (105, 106)])])
    assert required.len == 2
    assert not required.is_empty()
    assert pickle.loads(pickle.dumps(required)) == required

    selector = reat.selection.RequiredOrMismatches(required, mismatches)
    assert selector.required() == required
    assert selector.mismatches() == mismatches
    assert pickle.loads(pickle.dumps(selector)) == selector


def test_reat_sample_tags_only_need_equality():
    engine = reat.Reat(IndexedSources(REFERENCE), threads=1)
    layout = Layout.Single(Strandedness.Unstranded)
    engine.add_sources({"sample": 1}, [], layout)
    engine.add_sources({"sample": 2}, [], layout)

    results = engine.run([])

    assert [result.tag for result in results] == [{"sample": 1}, {"sample": 2}]


def test_reat_run_with_bam_fixture():
    assert BAM.exists()
    assert REFERENCE.exists()

    selector = reat.RequiredSites([("chr21", Orientation.Dual, [(3190, 3200)])])
    engine = reat.Reat(IndexedSources(REFERENCE), selector, threads=1)
    engine.add_sources("sample", [str(BAM)], Layout.Single(Strandedness.Unstranded))

    results = engine.run([reat.Task("chr21", [(3190, 3200)])])

    assert len(results) == 1
    assert results[0].tag == "sample"
    pileups = results[0].pileups()
    assert list(pileups.keys()) == [("chr21", Orientation.Dual)]
    task_pileup = pileups["chr21", Orientation.Dual]
    pileup = task_pileup.pileup()
    assert pileup.positions() == list(range(3190, 3200))
    assert pileup.counts().coverage() == [1] * 10
    assert len(task_pileup.reference()) == 10

    restored = pickle.loads(pickle.dumps(results[0]))
    assert restored.tag == "sample"
    restored_task_pileup = restored.pileups()["chr21", Orientation.Dual]
    restored_pileup = restored_task_pileup.pileup()
    assert restored_pileup.positions() == pileup.positions()
    assert restored_pileup.counts() == pileup.counts()
    assert restored_task_pileup.reference() == task_pileup.reference()
