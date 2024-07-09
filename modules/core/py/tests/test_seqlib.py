from biobit_core_py.seqlib import Strandedness, MatesOrientation, SeqLib


def test_seqlib():
    single = SeqLib.Single(Strandedness.Forward)
    assert single.strandedness == Strandedness.Forward, single.strandedness
    assert isinstance(single, SeqLib)
    assert isinstance(single, SeqLib.Single)

    paired = SeqLib.Paired(Strandedness.Forward, MatesOrientation.Inward)
    assert paired.strandedness == Strandedness.Forward, paired.strandedness
    assert paired.orientation == MatesOrientation.Inward, paired.orientation
    assert isinstance(paired, SeqLib)
    assert isinstance(paired, SeqLib.Paired)
