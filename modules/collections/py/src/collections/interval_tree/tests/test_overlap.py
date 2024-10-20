import pickle

from biobit.collections.interval_tree.overlap import Elements, Steps


def test_overlap_elements():
    empty = Elements()
    assert len(empty) == 0 and list(empty) == []
    assert empty.segments == [] and empty.elements == []
    assert Elements() == empty == Elements.from_existent([], [])
    assert pickle.loads(pickle.dumps(empty)) == empty

    segments = [[(-10, 10)], [], [(0, 5), (5, 10)]]
    elements = [["a"], [], ["b", "c"]]
    non_empty = Elements.from_existent(segments, elements)

    assert len(non_empty) == 3 and list(non_empty) == list(zip(segments, elements))
    assert non_empty.segments == segments and non_empty.elements == elements
    assert non_empty == Elements.from_existent(segments, elements)
    assert pickle.loads(pickle.dumps(non_empty)) == non_empty


def test_steps():
    elements = Elements.from_existent(
        [[(0, 20), (10, 20)], [], [(0, 100)], [(300, 400), (350, 400), (0, 500)]],
        [["a", "b"], [], ["c"], ["d", "e", "f"]],
    )

    steps = Steps()
    steps.build(elements, [(0, 30), (-100, -10), (0, 100), (300, 450)])
    print(list(steps))
    assert len(steps) == 4
    assert list(steps) == [
        [
            ((0, 10), {"a"}),
            ((10, 20), {"a", "b"}),
            ((20, 30), set()),
        ],
        [
            ((-100, -10), set()),
        ],
        [
            ((0, 100), {"c"}),
        ],
        [
            ((300, 350), {"d", "f"}),
            ((350, 400), {"d", "e", "f"}),
            ((400, 450), {"f"}),
        ],
    ]
