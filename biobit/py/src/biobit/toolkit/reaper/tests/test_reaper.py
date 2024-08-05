import pickle
from pathlib import Path

from biobit.core.ngs import Layout, Strandedness
from biobit.toolkit import reaper as rp

FILE = Path(__file__).resolve()


def test_workload():
    config = rp.Config(rp.model.RNAPileup(), rp.cmp.Enrichment(), rp.pcalling.ByCutoff(), rp.postfilter.NMS())
    assert pickle.loads(pickle.dumps(config)) == config

    workload = rp.Workload() \
        .add_region("1", 0, 1000, config) \
        .add_regions([("3", 0, 1000), ("2", 0, 1000)], config)

    assert pickle.loads(pickle.dumps(workload)) == workload


def test_ripper():
    config = rp.Config(rp.model.RNAPileup(), rp.cmp.Enrichment(), rp.pcalling.ByCutoff().set_cutoff(1.0),
                       rp.postfilter.NMS())
    workload = rp.Workload() \
        .add_region("1", 0, 100, config) \
        .add_regions([("2", 0, 10)], config)

    bam_1 = FILE.parent / "../../../../../../../resources/bam/A1+THP-1_mock_no-RNase_2.bam"
    assert bam_1.exists()

    bam_2 = FILE.parent / "../../../../../../../resources/bam/G2+Calu-3_SARS-CoV-2_RNase_3.bam"
    assert bam_2.exists()

    layout = Layout.Single(Strandedness.Reverse)

    bam_1, bam_2 = bam_1.as_posix(), bam_2.as_posix()

    ripped = rp.Reaper(threads=-23) \
        .add_source("Signal", bam_1, layout) \
        .add_sources("Control", [bam_1, bam_2], layout) \
        .add_comparison("Signal vs Control", "Signal", "Control", workload) \
        .add_comparison("Control vs Signal", "Control", "Signal", workload) \
        .run()

    for (ind, cmp) in enumerate(["Signal vs Control", "Control vs Signal"]):
        assert ripped[ind].comparison == cmp
        assert len(ripped[ind].regions) == 6

        for region in ripped[ind].regions:
            assert region.contig in ["1", "2"]
            assert region.segment.start == 0
            assert region.segment.end in [100, 10]
            assert region.modeled == [region.segment]
            assert len(region.raw_peaks) == 0
            assert len(region.filtered_peaks) == 0
