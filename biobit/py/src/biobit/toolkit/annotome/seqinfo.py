from itertools import chain
from typing import Iterable, Counter


class SeqInfo:
    def __init__(
            self, sequences: Iterable[str], length: dict[str, int], *,
            labels: dict[str, str] | None = None,
            description: dict[str, str] | None = None,
            synonyms: dict[str, Iterable[str]] | None = None
    ):
        cnts = Counter(sequences)
        repeated = [(name, cnt) for name, cnt in cnts.items() if cnt > 1]
        if repeated:
            raise ValueError(f"Repeated sequence names: {repeated}")
        self._sequences = sorted(cnts.keys())
        self._length = length
        missing = [x for x in self._sequences if x not in self._length]
        if missing:
            raise ValueError(f"Missing lengths for sequences: {missing}")
        self._labels = labels or {}
        self._description = description or {}

        synonyms = synonyms or {}
        self._synonyms = {}
        for seqname in self._sequences:
            syns = list(synonyms.get(seqname, [])) if synonyms else []
            syns.append(seqname)
            self._synonyms[seqname] = frozenset(syns)

        cnts = Counter(chain(self._synonyms.values()))
        repeated_syns = [(name, cnt) for name, cnt in cnts.items() if cnt > 1]
        if repeated_syns:
            raise ValueError(f"Repeated synonym names: {repeated_syns}")

    def sequences(self) -> Iterable[str]:
        """Return an iterable of all known sequence names, sorted alphabetically."""
        return self._sequences

    def length(self, name: str) -> int:
        """Return the length of the given sequence."""
        return self._length[name]

    def all_lengths(self) -> dict[str, int]:
        """Return a dictionary of all sequences and their lengths."""
        return self._length

    def label(self, name: str) -> str:
        """Return the label for the given sequence, or the name itself if no label is set."""
        return self._labels.get(name, name)

    def all_labels(self) -> dict[str, str]:
        return {name: self.label(name) for name in self._sequences}

    def description(self, name: str) -> str | None:
        """Return the description for the given sequence, or None if no description is set."""
        return self._description.get(name)

    def all_descriptions(self) -> dict[str, str]:
        """Return a dictionary of all sequences and their descriptions."""
        return self._description

    def synonyms(self, name: str) -> frozenset[str]:
        """Return the synonyms for the given sequence. Sequence name itself is always included."""
        return self._synonyms.get(name, frozenset())

    def all_synonyms(self) -> dict[str, frozenset[str]]:
        """Return a dictionary of all sequences and their synonyms."""
        return self._synonyms
