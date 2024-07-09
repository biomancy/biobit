import sys
from pathlib import Path
from subprocess import check_call


def fetch(query: str, url: str, saveto: Path, force: bool, verbose: bool):
    if saveto.exists() and not force:
        return

    if not saveto.name.endswith(".gz"):
        raise ValueError(
            f"Biomart cache must be stored in gzip-ed TSV files: expected {saveto} to end with .gz."
        )

    saveto.unlink(missing_ok=True)
    saveto.parent.mkdir(parents=True, exist_ok=True)

    query = "".join([x.strip() for x in query.split("\n")])

    url = f"{url}/biomart/martservice?query={query}"
    ungzipped = saveto.with_suffix("")

    try:
        stdout = sys.stdout if verbose else sys.stderr
        check_call(["wget", "-O", ungzipped, url], stdout=stdout)
        check_call(["gzip", ungzipped])
    except Exception as e:
        if ungzipped.exists():
            ungzipped.unlink()
        raise e
    assert saveto.exists()
