import os
import pickle
from pathlib import Path

import pytest

from biobit.io.fasta import Record, Reader, IndexedReader, Writer

RESOURCES = Path(os.environ['BIOBIT_RESOURCES']) / "fasta"


def test_fasta_record():
    record = Record("id", "seq")
    assert (record.id, record.seq) == ("id", "seq")

    # Check that the id and seq can be changed
    record.id = "new_id"
    record.seq = "newseq"
    assert (record.id, record.seq) == ("new_id", "newseq")

    # Check that the id and seq are validated
    for id in ["", "id\n"]:
        for seq in ["seq_1", "", "seq123"]:
            with pytest.raises(Exception):
                record.id = id
            with pytest.raises(Exception):
                record.seq = seq
            with pytest.raises(Exception):
                Record(id, seq)
    assert (record.id, record.seq) == ("new_id", "newseq")

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
        reader = Reader(RESOURCES / file)
        assert list(reader) == expected
        with pytest.raises(StopIteration):
            next(reader)

        # Creating a reader from a string path should work
        reader = Reader((RESOURCES / file).as_posix())
        buffer = Record("ID", "SEQ")
        for exp in expected:
            nxt = reader.read_record(into=buffer)
            assert nxt is buffer
            assert nxt == exp

        assert reader.read_record(into=buffer) is None

        reader = Reader(RESOURCES / file)
        assert reader.read_to_end() == expected


@pytest.mark.parametrize("path", ["indexed.fa", "indexed.fa.bgz"])
def test_indexed_fasta_reader(path):
    path = RESOURCES / path

    # Read all records in RAM
    reader = Reader(path)
    allrecords = {record.id: record.seq for record in reader}

    # Compare with indexed reader
    reader = IndexedReader(path.as_posix())
    assert reader.path == path

    # Create a reader from a pathlib.Path
    reader = IndexedReader(path)
    for id, seq in allrecords.items():
        assert reader.fetch_full_seq(id) == seq

        for start, end in [(0, 1), (10, 20), (10, len(seq))]:
            assert reader.fetch(id, (start, end)) == seq[start:end]

        for start, end in [(10, 2_000), (-1, 1), (10, len(seq) + 1)]:
            with pytest.raises(Exception):
                reader.fetch(id, (start, end))


@pytest.mark.parametrize("path", ["indexed.fa", "indexed.fa.bgz"])
def test_fasta_writer(path, tmp_path: Path):
    path = RESOURCES / path
    saveto = tmp_path / "output.fa"

    allrecords = Reader(path).read_to_end()

    with Writer(saveto, line_width=60) as writer:
        for record in allrecords:
            writer.write_record(record)
    assert Reader(saveto).read_to_end() == allrecords
    saveto.unlink()

    with Writer(saveto, line_width=120) as writer:
        writer.write_records(allrecords)
    assert Reader(saveto).read_to_end() == allrecords
