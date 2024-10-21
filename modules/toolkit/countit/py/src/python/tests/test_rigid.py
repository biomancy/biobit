from biobit.core.loc import Orientation
from biobit.core.ngs import Layout, Strandedness, MatesOrientation
from biobit.toolkit import countit


def test_countit():
    engine = countit.rigid.Engine.builder().set_threads(-1).add_elements([
        ("A", [("1", "+", [(1, 10), (11, 20)]), ("2", "-", [(10, 20), (10, 20)])]),
        (123, [("2", Orientation.Forward, [(0, 10)]), ("2", "=", [])])
    ]).add_partitions([
        ("1", (0, 248956422)),
        ("MT", (0, 16569)),
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
                    "Bam 1", "/home/alnfedorov/projects/biobit/resources/bam/A1+THP-1_mock_no-RNase_2.bam",
                    Layout.Paired(Strandedness.Reverse, MatesOrientation.Inward)
                )
            ],
            resolve
        )
        assert len(results) == 1
