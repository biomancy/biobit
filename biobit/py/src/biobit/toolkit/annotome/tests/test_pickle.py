import os
import pickle
import subprocess
import sys

from biobit.toolkit.annotome import Assembly, RefRegistry
from biobit.toolkit.annotome.annotation import Fasta
from biobit.toolkit.annotome.reference import Reference


def test_assembly_pickle_roundtrip():
    original = Assembly(
        "test",
        {"organism-c", "organism-a", "organism-b"},
        RefRegistry(
            [
                Reference("chr2", 200, "second", "description", {"two", "2", "II"}),
                Reference("chr1", 100, aliases={"one", "1", "I"}),
            ]
        ),
        {"fasta": Fasta(["genome.fa", "virus.fa"])},
    )
    restored = pickle.loads(pickle.dumps(original))
    assert restored.name == original.name
    assert restored.organisms == original.organisms
    assert list(restored.references.references()) == list(
        original.references.references()
    )
    assert restored.references.resolve("one") == original.references.resolve("chr1")
    assert (
        restored.annotations.fetch("fasta", Fasta).files
        == original.annotations.fetch("fasta", Fasta).files
    )


def test_assembly_pickle_is_stable_across_hash_seeds():
    code = """
import pickle
import sys
from biobit.toolkit.annotome import Assembly, RefRegistry
from biobit.toolkit.annotome.annotation import Fasta
from biobit.toolkit.annotome.reference import Reference

references = RefRegistry.merge([
    RefRegistry([Reference('chr1', 100, aliases={'one', '1', 'I'})]),
    RefRegistry([Reference('chr2', 200, aliases={'two', '2', 'II'})]),
])
assembly = Assembly(
    'test', {'organism-c', 'organism-a', 'organism-b'}, references,
    {key: Fasta(f'{key}.fa') for key in {'fasta', 'extra', 'other'}},
)
sys.stdout.buffer.write(pickle.dumps(assembly, protocol=3))
"""
    # Hash seeds are fixed at interpreter startup, so compare separate processes.
    pickles = [
        subprocess.run(
            [sys.executable, "-B", "-c", code],
            env=os.environ | {"PYTHONHASHSEED": seed},
            capture_output=True,
            check=True,
        ).stdout
        for seed in ("1", "2")
    ]
    assert pickles[0] == pickles[1]
