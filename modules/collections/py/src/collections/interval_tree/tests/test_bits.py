from biobit.collections.interval_tree import Bits, overlap


def test_bits():
    bits = Bits.builder().add([
        ((0, 5), "a"),
        ((5, 10), "b"),
        ((0, 10), "c"),
        ((10, 15), "d"),
    ]).addi((-100, 100), 2).build()

    oe = overlap.Elements()
    for x in [
        [(0, 10)], [(-1000, -999)], [(11, 200)], [], [(10, 20), (100, 110)]
    ]:
        bits.overlap(x, oe)

    print(list(oe))
    assert list(oe) == [
        ([(-100, 100), (0, 5), (0, 10), (5, 10)], [2, "a", "c", "b"]),
        ([], []),
        ([(-100, 100), (10, 15)], [2, "d"]),
        ([], []),
        ([(-100, 100), (10, 15)], [2, "d"]),
    ]

    oe.clear()
    assert oe == overlap.Elements()
