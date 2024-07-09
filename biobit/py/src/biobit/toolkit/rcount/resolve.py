from abc import ABC, abstractmethod
from collections import defaultdict
from collections.abc import Callable
from typing import TypeVar, Generic, cast

from biobit.ds.gindex import Overlap

__all__ = ["Resolution", "Proportional", "Binary", "Priority", "Normalize", "Counts"]

_T = TypeVar("_T")
_I = TypeVar("_I")
_O = TypeVar("_O")
_D = TypeVar("_D")

Counts = dict[_T, float]


class Resolution(Generic[_I, _O], ABC):
    # Resolution is required to turn a list of overlapping annotations into weights for one or more annotation categories.
    # This is necessary when a sequenced read/read pair isn't entirely within a known annotation 'type'.
    # All resolution strategies are chainable, so that multiple strategies can be applied in sequence.
    @abstractmethod
    def __call__(self, data: _I) -> _O:
        pass

    def apply(self, nxt: 'Callable[[_O], _T] | Resolution[_O, _T]') -> 'Resolution[_I, _T]':
        if isinstance(nxt, Resolution):
            return _Chain(self, nxt)
        elif isinstance(nxt, Callable):  # type: ignore # https://github.com/python/mypy/issues/6680
            return _Chain(self, _Lambda(nxt))
        else:
            raise TypeError(f"Cannot apply {nxt} to {self}")

    def __or__(self, nxt: 'Callable[[_O], _T] | Resolution[_O, _T]') -> 'Resolution[_I, _T]':
        return self.apply(nxt)


class _Chain(Generic[_I, _O, _T], Resolution[_I, _T]):
    def __init__(self, first: Resolution[_I, _O], second: Resolution[_O, _T]):
        self.first = first
        self.second = second

    def __call__(self, data: _I) -> _T:
        return self.second(self.first(data))


class _Lambda(Generic[_I, _O, _T], Resolution[_I, _O]):
    def __init__(self, fn: Callable[[_I], _O]):
        self.fn = fn

    def __call__(self, data: _I) -> _O:
        return self.fn(data)


class Proportional(Resolution[list[Overlap[_T]], Counts[_T | None]]):
    def __init__(self, empty: _T | None = None):
        self.empty = empty

    def __call__(self, overlaps: list[Overlap[_T]]) -> Counts[_T | None]:
        """
        The proportional strategy assigns weights to annotation categories based on the overlap of the target ROI
        with each category
        """
        if len(overlaps) == 0:
            return {self.empty: 1}

        weights: Counts[_T | None] = {}
        covered, total = 0, 0
        for roiov in overlaps:
            total += len(roiov.rng)  # original ROI length
            # step-wise ROI annotation
            for rng, annotations in roiov.to_steps():  # type: ignore # https://github.com/python/mypy/issues/16733
                length = len(rng)
                if not annotations:
                    covered += length
                    weights[self.empty] = weights.get(self.empty, 0) + length
                else:
                    covered += length
                    for anno in annotations:
                        weights[anno] = weights.get(anno, 0) + length

        assert covered == total, f"Covered length {covered} does not match total length {total}"
        return weights


class Binary(Resolution[list[Overlap[_T]], Counts[_T | None]]):
    def __init__(self, empty: _T | None = None):
        self.empty = empty

    def __call__(self, overlaps: list[Overlap[_T]]) -> Counts[_T | None]:
        """
        The binary strategy assigns weights to annotation categories based on the presence of at least a single base pair
        overlap between the read block and each category
        """
        categories: set[_T | None] = set()
        length, covered = 0, 0
        for blockov in overlaps:
            length += len(blockov.rng)
            for rng, step in blockov.to_steps():  # type: ignore # https://github.com/python/mypy/issues/16733
                if step:
                    covered += len(rng)
                    categories |= step

        if not categories:
            return {self.empty: 1}

        if covered != length:
            categories.add(self.empty)

        weights = {cat: 1.0 for cat in categories}
        return weights


class Priority(Generic[_T, _D], Resolution[Counts[_T], Counts[_T]]):
    def __init__(self, priority: list[_D], key: Callable[[_T], _D] | None = None):
        self.priority = priority
        self.key = key or (lambda x: cast(_D, x))

    def __call__(self, weights: Counts[_T]) -> Counts[_T]:
        """
        The priority strategy picks the first category in the priority list that is also present in the weights.
        If more than one category is mapped to the same priority, all such categories are returned.
        """
        keys: defaultdict[_D, list[_T]] = defaultdict(list)
        for cat in weights:
            keys[self.key(cat)].append(cat)

        categories: list[_T] | None = None
        for p in self.priority:
            if p in keys:
                categories = keys[p]
                break

        if categories is None:
            raise ValueError(f"None of the priority categories {self.priority} are present in the weights {weights}")

        return {cat: weights[cat] for cat in categories}


class Normalize(Resolution[Counts[_T], Counts[_T]]):
    def __init__(self, total: float = 1.0):
        self.total = total

    def __call__(self, weights: Counts[_T]) -> Counts[_T]:
        """
        Normalize annotation weights to sum to a given total
        """
        if len(weights) == 0:
            return weights

        total = sum(weights.values())
        if total == 0:
            raise ValueError(f"Total weight is 0, cannot normalize {weights} to {self.total}")

        factor = self.total / total
        return {k: v * factor for k, v in weights.items()}
