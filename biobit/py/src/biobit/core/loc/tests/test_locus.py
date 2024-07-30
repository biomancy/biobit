import pickle

import pytest

from biobit.core.loc import Locus, Segment, Orientation


def test_locus_new():
    for contig, start, end, orientation in [
        ("1", 1, 123, "+"), ("13", -10, 10, "-"), ("X", 0, 2, "=")
    ]:
        locus = Locus(contig, (start, end), orientation)
        assert locus.contig == contig
        assert locus.len() == end - start
        assert locus.segment == (start, end)
        assert locus.segment == Segment(start, end)
        assert locus.orientation == orientation
        assert locus.orientation == Orientation(orientation)

    for contig, start, end, orientation in [
        ("1", 123, 1, "+"), ("13", 10, -10, "-"), ("X", 0, 1, ".")
    ]:
        with pytest.raises(ValueError):
            Locus(contig, (start, end), orientation)


# def test_locus_fields():
#     segment = Segment(1, 123)
#     locus = Locus("1", segment, "+")
#     assert locus.segment is segment
#
#     segment.extend(1, 7)
#     assert locus.segment == (0, 130) and locus.segment is segment
#
#     orientation = locus.orientation
#     assert orientation is locus.orientation
#     assert orientation.flip() is not orientation and orientation == "-"
#     assert locus.orientation is not orientation and locus.orientation == "-"


def test_locus_flip():
    locus = Locus("1", (1, 123), "+")
    assert locus.flip() is locus and locus.orientation == "-"

    flipped = locus.flipped()
    assert flipped is not locus and flipped.orientation == "+"
    assert flipped.orientation is not locus.orientation and flipped.segment is not locus.segment

    flipped.orientation = "-"
    assert flipped is not locus and flipped.orientation == "-"
    assert flipped.orientation is not locus.orientation and flipped.segment is not locus.segment


# def test_locus_contains():
#     locus = Locus("1", (-10, 10), "+")
#     assert locus.contains(-10) and locus.contains(0)
#     assert not locus.contains(10) and not locus.contains(-11)
#
#
# def test_locus_intersects():
#     locus = Locus("1", (-10, 10), "+")
#     for segment in [
#         (-10, 0), (-10, -9), (-1, 3), (9, 10)
#     ]:
#         assert locus.intersects(("1", segment, "+"))
#         assert not locus.intersects(("1", segment, "-"))
#         assert not locus.intersects(("1", segment, "="))
#         assert not locus.intersects(("3", segment, "+"))
#
#     for segment in [
#         (-15, -13), (-11, -10), (10, 11), (13, 15)
#     ]:
#         assert not locus.intersects(("1", segment, "+"))
#         assert not locus.intersects(("1", segment, "-"))
#         assert not locus.intersects(("1", segment, "="))
#         assert not locus.intersects(("3", segment, "+"))

def test_locus_eq():
    locus = Locus("1", (1, 2), "+")
    assert locus == Locus("1", (1, 2), "+")
    assert locus == ("1", (1, 2), "+")

    assert locus != Locus("2", (1, 2), "+")
    assert locus != ("2", (1, 2), "+")

    assert locus != Locus("1", (1, 2), "=")
    assert locus != ("1", (1, 2), "=")


def test_locus_ord():
    loci = [
        Locus("1", (1, 2), "+"),
        Locus("1", (1, 2), "-"),
        Locus("1", (1, 2), "="),

        Locus("2", (1, 2), "+"),
        Locus("2", (1, 2), "-"),
        Locus("2", (1, 2), "="),

        Locus("1", (5, 6), "+"),
        Locus("1", (5, 6), "-"),
        Locus("1", (5, 6), "="),

        Locus("2", (5, 6), "+"),
        Locus("2", (5, 6), "-"),
        Locus("2", (5, 6), "="),

        Locus("1", (-2, -1), "+"),
        Locus("1", (-2, -1), "-"),
        Locus("1", (-2, -1), "="),

        Locus("2", (-2, -1), "+"),
        Locus("2", (-2, -1), "-"),
        Locus("2", (-2, -1), "="),
    ]
    loci.sort()

    assert loci == [
        Locus("1", (-2, -1), "-"),
        Locus("1", (1, 2), "-"),
        Locus("1", (5, 6), "-"),

        Locus("1", (-2, -1), "="),
        Locus("1", (1, 2), "="),
        Locus("1", (5, 6), "="),

        Locus("1", (-2, -1), "+"),
        Locus("1", (1, 2), "+"),
        Locus("1", (5, 6), "+"),

        Locus("2", (-2, -1), "-"),
        Locus("2", (1, 2), "-"),
        Locus("2", (5, 6), "-"),

        Locus("2", (-2, -1), "="),
        Locus("2", (1, 2), "="),
        Locus("2", (5, 6), "="),

        Locus("2", (-2, -1), "+"),
        Locus("2", (1, 2), "+"),
        Locus("2", (5, 6), "+"),
    ]


def test_pickle_locus():
    locus = Locus("1", (1, 123), "+")
    assert pickle.loads(pickle.dumps(locus)) == locus
    assert pickle.loads(pickle.dumps(locus)) == Locus("1", (1, 123), "+")
