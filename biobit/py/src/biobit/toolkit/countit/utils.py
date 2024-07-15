import pandas as pd

from .countit import Counts


def result_to_pandas(cnts: list[Counts]) -> tuple[pd.DataFrame, pd.DataFrame]:
    """Converts a list of Counts objects to a pair of pandas DataFrames.

    Args:
        cnts: A list of Counts objects.

    Returns:
        A pair of pandas DataFrames, the first one containing the counts and the second one containing the stats
    """
    allcounts, allstats = [], []
    for r in cnts:
        record = dict(zip(r.data, r.counts))
        record['source'] = r.source
        allcounts.append(record)

        for stat in r.stats:
            record = {
                "contig": stat.contig, "segment": stat.segment, "time_s": stat.time_s, "source": r.source,
                "inside_annotation": stat.inside_annotation, "outside_annotation": stat.outside_annotation
            }
            allstats.append(record)
    return pd.DataFrame(allcounts), pd.DataFrame(allstats)
