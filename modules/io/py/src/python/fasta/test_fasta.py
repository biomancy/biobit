import os
import pickle
from pathlib import Path

import pytest

from biobit.io.fasta import Record, Reader, IndexedReader

RESOURCES = Path(os.environ['BIOBIT_RESOURCES']) / "fasta"


def test_fasta_record():
    record = Record("id", "seq")
    assert record.id == "id"
    assert record.seq == "seq"

    # Check that the id and seq are read-only
    with pytest.raises(Exception):
        record.id = "new_id"
    with pytest.raises(Exception):
        record.seq = "new_seq"

    # Check that the id and seq are validated
    for id, seq in [
        ("", "seq"), ("id", ""), ("", ""), ("id\n", "seq"), ("id", "seq123")
    ]:
        with pytest.raises(Exception):
            Record(id, seq)

    pickled = pickle.loads(pickle.dumps(record))
    assert record == pickled
    assert record is not pickled
    assert record.id == pickled.id
    assert record.seq == pickled.seq


def test_fasta_reader():
    expected = [
        Record(" My Super ЮТФ-последовательность Прямо Here   ", "NonUniformLinesAreAllowed"),
        Record("	Another UTF sequence with tabs and spaces	", "AnySequenceWithoutSpacesAllowedHere"),
    ]

    for file in "example.fa", "example.fa.gz":
        reader = Reader((RESOURCES / file).as_posix())
        assert list(reader) == expected
        with pytest.raises(StopIteration):
            next(reader)

        reader = Reader((RESOURCES / file).as_posix())
        buffer = Record("ID", "SEQ")
        for exp in expected:
            nxt = reader.read_record(into=buffer)
            assert nxt is buffer
            assert nxt == exp

        assert reader.read_record(into=buffer) is None


@pytest.mark.parametrize("path", ["indexed.fa", "indexed.fa.gz"])
def test_indexed_fasta_reader(path):
    path = (RESOURCES / path).as_posix()

    # Read all records in RAM
    reader = Reader(path)
    allrecords = {record.id: record.seq for record in reader}

    # Compare with indexed reader
    reader = IndexedReader(path)

    for id, seq in allrecords.items():
        assert reader.fetch_full_seq(id) == seq

        for start, end in [(0, 1), (10, 20), (10, len(seq))]:
            assert reader.fetch(id, (start, end)) == seq[start:end]

        for start, end in [(10, 2_000), (-1, 1), (10, len(seq) + 1)]:
            with pytest.raises(Exception):
                reader.fetch(id, (start, end))
