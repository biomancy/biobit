import os
from pathlib import Path

import pytest
from biobit.core.loc import Orientation
from biobit.io.bam import Reader
from biobit.toolkit import countit
from biobit.toolkit.bioproj import Layout, MatesOrientation, Strandedness


def test_countit():
    engine = (
        countit.rigid.Engine.builder()
        .set_threads(-1)
        .add_elements(
            [
                (
                    "A",
                    [
                        ("chr1", "+", [(1, 10), (11, 20)]),
                        ("chr2", "-", [(10, 20), (10, 20)]),
                    ],
                ),
                (123, [("chr2", Orientation.Forward, [(0, 10)]), ("chr2", "=", [])]),
            ]
        )
        .add_partitions(
            [
                ("chr1", (0, 248956422)),
                ("chrM", (0, 16569)),
            ]
        )
        .build()
    )

    resolutions = [
        countit.rigid.resolution.AnyOverlap(),
        countit.rigid.resolution.OverlapWeighted(),
        countit.rigid.resolution.TopRanked(["A", 123]),
    ]

    path = Path(
        "/home/alnfedorov/projects/biobit/resources/bam/G1+THP-1_EMCV_no-RNase_3.markdup.sorted.bam"
    )
    if path.exists():
        for resolve in resolutions:
            results = engine.run(
                [
                    (
                        "Bam 1",
                        str(path),
                        Layout.Paired(Strandedness.Reverse, MatesOrientation.Inward),
                    )
                ],
                resolve,
            )
            assert len(results) == 1


def test_worker_errors_reach_python():
    engine = (
        countit.rigid.Engine.builder()
        .set_threads(2)
        .add_partitions([("missing-contig", (0, 1))])
        .build()
    )
    # Only the header/index are opened; the failed query reads no alignments.
    path = Path(os.environ["BIOBIT_RESOURCES"]) / "bam" / "RNA-seq.CHM13v2.21-22.bam"
    source = Reader(path, inflags=3, exflags=2572)
    layout = Layout.Paired(Strandedness.Unstranded, MatesOrientation.Inward)
    with pytest.raises(
        RuntimeError, match="Unknown BAM reference sequence: missing-contig"
    ) as error:
        engine.run([("sample", source, layout)], countit.rigid.resolution.AnyOverlap())
    assert "source 0, partition 0" in str(error.value)
