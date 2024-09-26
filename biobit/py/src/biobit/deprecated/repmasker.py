from pathlib import Path

import pandas as pd


class RepmaskerClassification:
    def __init__(self, path: Path):
        self.path = path
        self._mapping: dict[str, tuple[str, str | None, str]] = {}

        data = pd.read_csv(path, sep="\t").replace(pd.NA, None)
        for name, family, cls in data[["name", "family", "class"]].itertuples(index=False, name=None):
            assert name and name not in self._mapping, name
            self._mapping[name] = (name, family, cls)

    def classify(self, repname: str) -> tuple[str, str | None, str] | None:
        return self._mapping.get(repname, None)

    def names(self) -> set[str]:
        return set(x[0] for x in self._mapping.values())

    def families(self) -> set[str]:
        return set(x[1] for x in self._mapping.values() if x[1] is not None)

    def classes(self) -> set[str]:
        return set(x[2] for x in self._mapping.values())
