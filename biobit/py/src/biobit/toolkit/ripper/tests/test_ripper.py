from pathlib import Path

from biobit.core.ngs import Layout, Strandedness
from biobit.toolkit.ripper import Config, Ripper

FILE = Path(__file__).resolve()


def test_ripper():
    config = Config() \
        .with_control_baseline(0.5) \
        .with_min_raw_signal(1e6) \
        .with_pcalling_params(10, 10, 0.5) \
        .with_scaling_factors(0.5, 2)

    layout = Layout.Single(Strandedness.Forward)

    bam_1 = FILE.parent / "../../../../../../../resources/bam/A1+THP-1_mock_no-RNase_2.bam"
    assert bam_1.exists()

    bam_2 = FILE.parent / "../../../../../../../resources/bam/G2+Calu-3_SARS-CoV-2_RNase_3.bam"
    assert bam_2.exists()

    bam_1, bam_2 = bam_1.as_posix(), bam_2.as_posix()

    ripped = Ripper(threads=-23) \
        .add_partition("1", 1, 1000) \
        .add_source("Signal", bam_1, layout) \
        .add_sources("Control", [bam_1, bam_2], layout) \
        .add_comparison("Signal vs Control", "Signal", "Control", config) \
        .add_comparison("Control vs Signal", "Control", "Signal", config) \
        .run()

    for (ind, cmp) in enumerate(["Signal vs Control", "Control vs Signal"]):
        assert ripped[ind].tag == cmp
        assert len(ripped[ind].regions) == 1

        region = ripped[ind].regions[0]
        assert region.contig == "1" and region.segment.start == 1 and region.segment.end == 1000

        peaks = region.peaks
        assert len(peaks.forward) == 0 and len(peaks.reverse) == 0 and len(peaks.dual) == 0
