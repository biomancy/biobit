import pickle
from collections import defaultdict
from pathlib import Path
from typing import Callable

import pandas as pd

from .transcriptome import GeneBundle, RNABundle, CDSBundle, Location


class Annotome[AttrGene, AttrRNA, AttrCDS]:
    def __init__(
            self, assembly: str, source: str,
            genes: GeneBundle[AttrGene],
            rnas: RNABundle[AttrRNA],
            cds: CDSBundle[AttrCDS],
    ):
        self.assembly: str = assembly
        self.source: str = source
        self.genes: GeneBundle[AttrGene] = genes
        self.rnas: RNABundle[AttrRNA] = rnas
        self.cds: CDSBundle[AttrCDS] = cds

        # Validate that all RNA transcripts are associated with a gene
        inferred_parents = defaultdict(set)
        for rna in self.rnas.values():
            if rna.gene not in self.genes:
                raise ValueError(f"RNA transcript {rna.ind} is associated with unknown gene {rna.gene}")
            inferred_parents[rna.gene].add(rna.ind)

        for gid, tids in inferred_parents.items():
            if self.genes[gid].transcripts != tids:
                raise ValueError(f"Gene {gid} has mismatched RNA transcripts: {tids} vs {self.genes[gid].transcripts}")


def read_pkl[AttrGene, AttrRNA, AttrORF](path: str) -> Annotome[AttrGene, AttrRNA, AttrORF]:
    with open(path, "rb") as f:
        return pickle.load(f)


def preprocess_gff(
        path: Path, ignore_sources: set[str], ignore_types: set[str],
        hook: Callable[
            [str, Location, dict[str, str]], tuple[str, Location, dict[str, str]] | None
        ] = lambda x, y, z: (x, y, z),
        ind_key: Callable[[str, dict[str, str]], str] = lambda _, x: x["ID"]
) -> dict[str, dict[str, list[tuple[Location, str, dict[str, str]]]]]:
    df = pd.read_csv(
        path, sep='\t', comment="#",
        names=["seqid", "source", "type", "start", "end", "score", "strand", "phase", "attrs"],
        dtype={
            "seqid": str, "source": str, "type": str, "start": int, "end": int,
            "score": str, "strand": str, "phase": str, "attrs": str
        }
    ).drop(columns=["score", "phase"])

    ind2type: dict[str, str] = {}
    records: dict[str, dict[str, list[tuple[Location, str, dict[str, str]]]]] = defaultdict(lambda: defaultdict(list))
    for seqid, source, type, start, end, strand, attributes in df.itertuples(index=False, name=None):
        if source in ignore_sources or type in ignore_types:
            continue

        start -= 1
        loc = Location(seqid, strand, start, end)
        attributes = dict(x.split("=", maxsplit=1) for x in attributes.split(";"))

        posthook = hook(type, loc, attributes)
        if posthook is None:
            continue
        type, loc, attributes = posthook

        ind = ind_key(type, attributes)
        if ind2type.setdefault(ind, type) != type:
            raise ValueError(f"Inconsistent type for {ind}: {ind2type[ind]} vs {type}")

        records[type][ind].append((loc, source, attributes))
    return {k: dict(v) for k, v in records.items()}
