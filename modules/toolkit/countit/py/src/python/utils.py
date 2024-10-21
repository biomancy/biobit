from collections import defaultdict
from collections.abc import Iterable
from typing import Any, Callable

import pandas as pd

from biobit.core.loc import Segment, Orientation, IntoOrientation
from .result import Counts


def result_to_pandas[S, E](cnts: list[Counts[S, E]]) -> tuple[pd.DataFrame, pd.DataFrame]:
    """
    Converts a list of Counts objects to a pair of pandas DataFrames.

    Args:
        cnts: A list of Counts objects.

    Returns:
        A pair of pandas DataFrames, the first one containing the counts and the second one containing the stats.
    """
    allcounts, allstats = [], []
    for r in cnts:
        record: dict[Any, Any] = dict(zip(r.elements, r.counts))
        record['source'] = r.source
        allcounts.append(record)

        for metric in r.partitions:
            record = {
                "contig": metric.contig, "segment": metric.segment, "time_s": metric.time_s, "source": r.source,
                "resolved": metric.outcomes.resolved, "discarded": metric.outcomes.discarded
            }
            allstats.append(record)
    return pd.DataFrame(allcounts), pd.DataFrame(allstats)


def resolve_annotation(
        annotation: dict[Any, dict[tuple[str, IntoOrientation], list[Segment]]],
        resolution: Callable[[str, Orientation, int, int, set[Any]], Iterable[Any]]
) -> dict[Any, dict[tuple[str, Orientation], list[Segment]]]:
    """
    Statically resolve overlapping annotation regions.

    Args:
        annotation: A dictionary where the key is an annotation key, and the value is another dictionary.
                    The inner dictionary maps a tuple of (contig, orientation) to a list of Segment objects.
        resolution: A callable that accepts the coordinates of a region (contig, orientation, start, end, keys) and
                    all annotation keys inside the region. It should return an iterable of resolved keys.

    Returns:
        A dictionary where the key is a resolved annotation key, and the value is another dictionary.
        The inner dictionary maps a tuple of (contig, orientation) to a list of resolved Segment objects.
    """
    # Group all annotation items per contig and strand
    groups: dict[tuple[str, Orientation], list[tuple[Segment, Any]]] = defaultdict(list)
    for key, anno in annotation.items():
        for (contig, orientation), regions in anno.items():
            orientation = Orientation(orientation)
            for region in regions:
                groups[(contig, orientation)].append((region, key))

    # Resolve each group
    resolved: dict[Any, dict[tuple[str, Orientation], list[Segment]]] = defaultdict(lambda: defaultdict(list))
    for (contig, orientation), reggroup in groups.items():
        reggroup = sorted(reggroup, key=lambda x: x[0].start)

        start, end = reggroup[0][0].start, reggroup[0][0].end
        cache = []

        ind = 0
        cursor = reggroup[ind][0]
        while True:
            if cursor.start == start:
                # Add and shrink the window to the left
                end = min(end, cursor.end)
                cache.append(reggroup[ind])

                ind += 1
                if ind >= len(reggroup):
                    break
                cursor = reggroup[ind][0]
            elif cursor.start < end:
                # Shrink the current window to the left and don't consume the next region
                end = min(end, cursor.start)
            else:
                # Next region is outside the current cache => process the cache and start a new one
                assert cursor.start >= end

                for r in resolution(contig, orientation, start, end, {x[1] for x in cache}):
                    resolved[r][(contig, orientation)].append(Segment(start, end))
                cache = [x for x in cache if x[0].end > end]

                if cache:
                    start = end
                    end = min(x[0].end for x in cache)
                elif end == cursor.start:
                    start = cursor.start
                    end = cursor.end
                else:
                    start = end
                    end = cursor.start

        # Resolve the leftover cache
        while cache:
            for r in resolution(contig, orientation, start, end, {x[1] for x in cache}):
                resolved[r][(contig, orientation)].append(Segment(start, end))
            cache = [x for x in cache if x[0].end > end]

            if cache:
                start = end
                end = min(x[0].end for x in cache)

    # Default dict to dict
    result = {}
    for k, v in resolved.items():
        result[k] = dict(v)

    return result
