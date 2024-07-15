from biobit.core.loc import Orientation
from biobit.toolkit.countit import CountIt


def test_count_it():
    countit = CountIt(threads=-23)

    countit \
        .add_annotation("A", [("1", "+", [(1, 10), (11, 20)]), ("2", "-", [(10, 20), (10, 20)])]) \
        .add_annotation(123, [("2", Orientation.Forward, [(0, 10)]), ("2", "=", [])])

    results = countit.run()
    assert len(results) == 0
