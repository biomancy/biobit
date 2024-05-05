import gzip
from pathlib import Path


class RepmaskerClassification:
    def __init__(self, path: Path):
        self.path = path
        self._mapping: dict[str, tuple[str, str, str]] = {}

        with gzip.open(path, "rt") as stream:
            for line in stream:
                line = line.strip()
                if len(line) == 0:
                    continue
                name, cls, family = line.split()
                assert name not in self._mapping, name
                self._mapping[name] = (name, family, cls)

    def classify(self, repname: str) -> tuple[str, str, str] | None:
        return self._mapping.get(repname, None)

    def names(self) -> set[str]:
        return set(x[0] for x in self._mapping.values())

    def families(self) -> set[str]:
        return set(x[1] for x in self._mapping.values())

    def classes(self) -> set[str]:
        return set(x[2] for x in self._mapping.values())
