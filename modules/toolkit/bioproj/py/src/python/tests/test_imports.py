import subprocess
import sys
from textwrap import dedent


def test_public_native_modules_import_in_a_fresh_interpreter() -> None:
    result = subprocess.run(
        [
            sys.executable,
            "-c",
            dedent(
                """
                import pickle
                from importlib import import_module
                from importlib.util import find_spec

                # Import a leaf first, before any explicit parent import.
                from biobit.toolkit.bioproj.io.json import read
                from biobit.toolkit import bioproj

                for suffix in (
                    '', '.data', '.data.asset', '.data.dataset', '.design',
                    '.io', '.io.json', '.provenance', '.provenance.acquisition',
                    '.provenance.library',
                ):
                    public_name = 'biobit.toolkit.bioproj' + suffix
                    module = import_module(public_name)
                    assert module is import_module('biobit.rs.toolkit.bioproj' + suffix)
                    parent_name, _, child_name = public_name.rpartition('.')
                    assert getattr(import_module(parent_name), child_name) is module
                    assert module.__name__ == public_name
                    assert find_spec(public_name).name == public_name
                    for name in module.__all__:
                        value = getattr(module, name)
                        if name in {
                            "MetaValue", "Asset", "AssetId", "Dataset", "DatasetId",
                            "Design", "DesignId", "StrPath", "Acquisition",
                            "AcquisitionId", "Library", "LibraryId", "P5P7Input",
                        }:
                            assert pickle.loads(pickle.dumps(value)) == value

                assert read is bioproj.io.json.read
                """
            ),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
