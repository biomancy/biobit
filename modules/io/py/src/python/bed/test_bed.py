import os
from pathlib import Path

import pytest

from biobit.core.loc import IntoInterval, IntoOrientation, Orientation, Interval
from biobit.io.bed import Bed3, Bed4, Bed5, Bed6, Bed8, Bed9, Bed12, Reader
from biobit.test_utils import ensure_pickable

RESOURCES = Path(os.environ['BIOBIT_RESOURCES']) / "bed"


def _bed3(obj, seqid: str, interval: IntoInterval | None):
    assert obj.seqid == seqid
    obj.seqid = "seqid"
    assert obj.seqid == "seqid"

    if interval is not None:
        interval = Interval(*interval) if isinstance(interval, tuple) else interval
        obj.interval = obj.interval.extended(left=1, right=1)
        assert obj.interval == (interval.start - 1, interval.end + 1)

    prev = obj.interval
    obj.set(seqid="seqid_new", interval=None)
    assert (obj.seqid, obj.interval) == ("seqid_new", prev)

    # Only _ and alphanumeric strings with less than 255 characters are allowed
    for seqid in ["", "seqid?", "seq$id", "seq-id", "seq id", "seq.id", "A" * 256]:
        with pytest.raises(Exception):
            obj.seqid = seqid

    # Only non-negative intervals are allowed
    for interval in [(-1, 10), (-10, 0), (-2, -1)]:
        with pytest.raises(Exception):
            obj.interval = interval


def _bed4(obj, seqid: str, interval: IntoInterval | None, name: str):
    _bed3(obj, seqid, interval)

    assert obj.name == name
    obj.name = "super-name"
    assert obj.name == "super-name"

    obj.set(name="super-name-new", seqid="_A_")
    assert (obj.name, obj.seqid) == ("super-name-new", "_A_")

    # Only [\x20-\x7e]{1,255} characters are allowed for the name
    for name in ["", "A" * 256]:
        with pytest.raises(Exception):
            obj.name = name


def _bed5(obj, seqid: str, interval: IntoInterval | None, name: str, score: int):
    _bed4(obj, seqid, interval, name)

    assert obj.score == score
    obj.score = 100
    assert obj.score == 100

    obj.set(score=1000, name="bed5?name")
    assert (obj.score, obj.name) == (1000, "bed5?name")

    # Scores must be in the range [0, 1000]
    for score in [-1, 1001]:
        with pytest.raises(Exception):
            obj.score = score


def _bed6(obj, seqid: str, interval: IntoInterval | None, name: str, score: int, orientation: IntoOrientation):
    _bed5(obj, seqid, interval, name, score)

    assert obj.orientation == orientation
    obj.orientation = "+"
    assert obj.orientation == "+"

    obj.set(orientation="-", score=123)
    assert (obj.orientation, obj.score) == (Orientation.Reverse, 123)

    # Only "+", "-", and "=" are allowed for the orientation
    for orientation in ["", "++", "--", "."]:
        with pytest.raises(Exception):
            obj.orientation = orientation


def _bed8(obj, seqid: str, _: IntoInterval | None, name: str, score: int, orientation: IntoOrientation,
          thick: IntoInterval | None):
    _bed6(obj, seqid, None, name, score, orientation)

    if thick:
        assert obj.thick == thick
        # We can change both the thick interval and the main interval to retain the invariant
        obj.set(interval=(100, 200), thick=(105, 150))
        assert (obj.interval, obj.thick) == ((100, 200), (105, 150))

        # But changing either of them to break the invariant should raise an exception
        with pytest.raises(Exception):
            obj.interval = (100, 110)

        with pytest.raises(Exception):
            obj.thick = (90, 150)

    # Thick intervals must be inside the main interval
    start, end = obj.interval.start, obj.interval.end
    for thick in [(start - 1, end), (start, end + 1), (start - 1, end + 1)]:
        with pytest.raises(Exception):
            obj.thick = thick

    obj.thick = obj.interval


def _bed9(obj, seqid: str, interval: IntoInterval | None, name: str, score: int, orientation: IntoOrientation,
          thick: IntoInterval | None, rgb: tuple[int, int, int]):
    _bed8(obj, seqid, interval, name, score, orientation, thick)

    assert obj.rgb == rgb
    obj.rgb = (255, 0, 0)
    assert obj.rgb == (255, 0, 0)

    obj.set(rgb=(0, 255, 0), name="RGB rules")
    assert (obj.rgb, obj.name) == ((0, 255, 0), "RGB rules")

    # RGB values must be in the range [0, 255]
    for rgb in [(-1, 0, 0), (0, -1, 0), (0, 0, -1), (256, 0, 0), (255, 256, 0), (0, 0, 256)]:
        with pytest.raises(Exception):
            obj.rgb = rgb


def _bed12(obj, seqid: str, _interval: IntoInterval, name: str, score: int, orientation: IntoOrientation,
           _thick: IntoInterval, rgb: tuple[int, int, int], blocks: list[IntoInterval]):
    assert obj.blocks == blocks

    # Drop blocks to avoid invariant violation when modifying the interval or thick
    _bed9(obj, seqid, None, name, score, orientation, None, rgb)

    # Blocks must be non-overlapping, sorted, and start/end on the main interval boundaries
    # Note: blocks are always in local coordinates
    for blocks in [
        [(0, 100)], [(0, 5), (99, 100)], [(x, x + 1) for x in range(100)]
    ]:
        obj.set(interval=(1, 101), thick=(5, 100), blocks=blocks)
        assert (obj.interval, obj.thick, obj.blocks) == ((1, 101), (5, 100), blocks)


def test_bed_records():
    fields = ["seqid"]
    values = ["NAME"]

    for cls, fn, field, val in [
        (Bed3, _bed3, "interval", (1, 10)),
        (Bed4, _bed4, "name", "MySuper - _ Name"),
        (Bed5, _bed5, "score", 100),
        (Bed6, _bed6, "orientation", "+"),
        (Bed8, _bed8, "thick", (3, 5)),
        (Bed9, _bed9, "rgb", (0, 0, 0)),
        (Bed12, _bed12, "blocks", [(0, 2), (3, 4), (5, 9)]),
    ]:
        fields.append(field)
        values.append(val)

        print(list(zip(fields, values)))
        record = cls(*values)
        assert record == cls(*values)

        fn(record, *values)
        ensure_pickable(record, *fields)


def test_bed_reader():
    expected = [
        Bed12(
            "12", (100171448, 100171534), "1064+]", 0, "+",
            (100171448, 100171534), (0, 0, 0), [(0, 86)]
        ),
        Bed12(
            "13", (31643773, 31646400), "204+]", 13, "=",
            (31643773, 31646400), (0, 0, 255), [(0, 250), (2185, 2627)]
        ),
        Bed12(
            "17", (38362989, 38379729), "668+]", 98, "=",
            (38362989, 38379729), (0, 0, 0), [
                (0, 166), (1894, 1981), (3152, 3398), (10024, 10141), (12682, 12807), (13258, 13375), (15678, 16740)
            ]
        ),
        Bed12(
            "6", (137457714, 137460096), "129 -]", 1000, "-",
            (137457714, 137460096), (0, 255, 0), [(0, 2382)]
        ),
    ]

    for file in "example.bed", "example.bed.gz":
        reader = Reader.bed12(RESOURCES / file)
        assert list(reader) == expected
        with pytest.raises(StopIteration):
            next(reader)

        # Creating a reader from a string path should work
        reader = Reader.bed12((RESOURCES / file).as_posix())
        buffer = Bed12.default()

        for exp in expected:
            nxt = reader.read_record(into=buffer)
            assert nxt is buffer
            assert nxt == exp
        assert reader.read_record() is None

        reader = Reader.bed12(RESOURCES / file)
        assert reader.read_to_end() == expected
