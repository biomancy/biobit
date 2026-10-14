import pickle
from pathlib import Path

import pytest
from biobit.toolkit import reaper as rp
from biobit.toolkit.bioproj import Layout, MatesOrientation, Strandedness

FILE = Path(__file__).resolve()


def test_workload():
    config = rp.Config(
        rp.model.RNAPileup(),
        rp.cmp.Enrichment(),
        rp.pcalling.ByCutoff(),
        rp.postfilter.NMS(),
    )
    assert pickle.loads(pickle.dumps(config)) == config

    workload = (
        rp.Workload()
        .add_region("1", 0, 1000, config)
        .add_regions([("3", 0, 1000), ("2", 0, 1000)], config)
    )

    assert pickle.loads(pickle.dumps(workload)) == workload


def test_ripper():
    config = rp.Config(
        rp.model.RNAPileup(),
        rp.cmp.Enrichment(),
        rp.pcalling.ByCutoff().set_cutoff(1.0),
        rp.postfilter.NMS(),
    )
    workload = (
        rp.Workload()
        .add_region("chr1", 0, 100, config)
        .add_regions([("chr2", 0, 10)], config)
    )

    bam_1 = (
        FILE.parent
        / "../../../../../../../resources/bam/G1+THP-1_EMCV_no-RNase_3.markdup.sorted.bam"
    )
    bam_2 = (
        FILE.parent
        / "../../../../../../../resources/bam/F1+THP-1_EMCV_RNase_3.markdup.sorted.bam"
    )
    if not bam_1.exists() or not bam_2.exists():
        return

    layout = Layout.Single(Strandedness.Reverse)

    bam_1, bam_2 = bam_1.as_posix(), bam_2.as_posix()

    ripped = (
        rp.Reaper(threads=-23)
        .add_source("Signal", bam_1, layout)
        .add_sources("Control", [bam_1, bam_2], layout)
        .add_comparison("Signal vs Control", "Signal", "Control", workload)
        .add_comparison("Control vs Signal", "Control", "Signal", workload)
        .run()
    )

    for ind, cmp in enumerate(["Signal vs Control", "Control vs Signal"]):
        assert ripped[ind].comparison == cmp
        assert len(ripped[ind].regions) == 0


@pytest.mark.parametrize(
    "layout",
    [
        Layout.Single(Strandedness.Forward),
        Layout.Paired(Strandedness.Reverse, MatesOrientation.Inward),
    ],
    ids=["single", "paired"],
)
def test_reaper_rejects_empty_source_lists(layout: Layout) -> None:
    # PyO3 extracts layout before entering add_sources. Reaching this domain
    # error proves that both subclasses are accepted as native PyLayout inputs;
    # no BAM readers or processing are needed to exercise the API boundary.
    with pytest.raises(
        RuntimeError, match="At least one source is required for a sample"
    ):
        rp.Reaper(threads=1).add_sources("Empty", [], layout)
