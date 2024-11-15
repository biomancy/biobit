from biobit.core.loc import Orientation
from biobit.core.ngs import Layout, Strandedness, MatesOrientation
from biobit.toolkit import countit


def test_countit():
    engine = countit.rigid.Engine.builder().set_threads(-1).add_elements([
        ("A", [("chr1", "+", [(1, 10), (11, 20)]), ("chr2", "-", [(10, 20), (10, 20)])]),
        (123, [("chr2", Orientation.Forward, [(0, 10)]), ("chr2", "=", [])])
    ]).add_partitions([
        ("chr1", (0, 248956422)),
        ("chrM", (0, 16569)),
    ]).build()

    resolutions = [
        countit.rigid.resolution.AnyOverlap(),
        countit.rigid.resolution.OverlapWeighted(),
        countit.rigid.resolution.TopRanked(["A", 123])
    ]

    for resolve in resolutions:
        results = engine.run(
            [
                (
                    "Bam 1", "/home/alnfedorov/projects/biobit/resources/bam/G1+THP-1_EMCV_no-RNase_3.markdup.sorted.bam",
                    Layout.Paired(Strandedness.Reverse, MatesOrientation.Inward)
                )
            ],
            resolve
        )
        assert len(results) == 1
