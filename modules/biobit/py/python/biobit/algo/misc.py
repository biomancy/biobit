from collections.abc import Iterable
from typing import TypeVar, Callable

T = TypeVar("T")


def group_within(items: list[T], *, distance: Callable[[T, T], float], maxdist: int) -> Iterable[Iterable[T]]:
    result: list[list[T]] = []
    if len(items) == 0:
        return result

    cache = [items[0]]
    for p in items[1:]:
        dist = min(distance(c, p) for c in cache)
        if dist >= maxdist:
            result.append(cache)
            cache = [p]
        else:
            cache.append(p)
    assert len(cache) >= 1
    result.append(cache)
    return result
