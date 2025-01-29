from biobit.toolkit import repeto


def test_optimize_complex():
    def do_test(dsrna, optscore, expected):
        scores, dsrna = zip(*dsrna)
        dsrna = [repeto.repeats.InvRepeat([repeto.repeats.InvSegment(left, right) for left, right in d]) for d in dsrna]

        predicted, score = repeto.optimize.run(dsrna, scores)
        assert score == optscore and len(predicted) == len(expected), (score, optscore, predicted, expected)
        predicted = sorted(predicted, key=lambda x: x.brange())

        for p, exp in zip(predicted, expected):
            assert p is dsrna[exp], (p.segments, dsrna[exp].segments)

    dsrna = [
        (1, [[(0, 2), (38, 40)], [(3, 5), (35, 37)], [(6, 8), (32, 34)]]),
        (1, [[(9, 12), (28, 31)], [(13, 14), (26, 27)]]),
        (1, [[(2, 3), (4, 5)]]),
        (1, [[(7, 8), (12, 13)]]),
        (1, [[(16, 20), (21, 25)]]),
        (1, [[(27, 30), (34, 37)]])
    ]
    do_test(dsrna, 4, [2, 3, 4, 5])

    dsrna = [
        (3, [[(0, 2), (38, 40)], [(3, 5), (35, 37)], [(6, 8), (32, 34)]]),
        (3, [[(9, 12), (28, 31)], [(13, 14), (26, 27)]]),
        (1, [[(2, 3), (4, 5)]]),
        (1, [[(7, 8), (12, 13)]]),
        (2, [[(16, 20), (21, 25)]]),
        (1, [[(27, 30), (34, 37)]])
    ]
    do_test(dsrna, 8, [0, 1, 4])
