from pathlib import Path

import pytest

from biobit.io.bam import Reader

RESOURCES = Path(__file__).parent / "resources"


def test_bam_reader():
    file = RESOURCES / "G1+THP-1_EMCV_no-RNase_3.markdup.sorted.bam"
    for fname in file, str(file), file.as_posix():
        reader = Reader(fname)
        assert reader is not None
        assert reader.filename == str(file)
        assert reader.inflags == 0
        assert reader.exflags == 516
        assert reader.minmapq == 0
        assert reader.batch_size == 1024

    reader = Reader(file, 2, 24, 255, 32)
    assert reader is not None
    assert reader.filename == str(file)
    assert reader.inflags == 2
    assert reader.exflags == 24
    assert reader.minmapq == 255
    assert reader.batch_size == 32

    another = Reader(file, inflags=2, exflags=24, minmapq=255, batch_size=32)
    assert reader == another

    for kwargs in [
        {"inflags": -1}, {"exflags": 1_000_000}, {"minmapq": -1}, {"minmapq": 312}
    ]:
        with pytest.raises(OverflowError):
            Reader(file, **kwargs)
