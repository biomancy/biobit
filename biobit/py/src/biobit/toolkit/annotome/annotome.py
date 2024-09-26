import pickle
from collections import defaultdict
from collections.abc import Mapping
from typing import Hashable

from .gene import GeneBundle
from .rna import RNABundle


class Annotome[GeneAttrs: Mapping, GeneTag: Hashable, RNAAttrs: Mapping, RNATag: Hashable]:
    def __init__(
            self, assembly: str, source: str, genes: GeneBundle[GeneAttrs, GeneTag], rnas: RNABundle[RNAAttrs, RNATag]
    ):
        self.assembly: str = assembly
        self.source: str = source
        self.genes: GeneBundle[GeneAttrs, GeneTag] = genes
        self.rnas: RNABundle[RNAAttrs, RNATag] = rnas

        # Validate that all RNA transcripts are associated with a gene
        inferred_parents = defaultdict(set)
        for rna in self.rnas.values():
            if rna.gene not in self.genes:
                raise ValueError(f"RNA transcript {rna.id} is associated with unknown gene {rna.gene}")
            inferred_parents[rna.gene].add(rna.id)

        for gid, tids in inferred_parents.items():
            if self.genes[gid].transcripts != tids:
                raise ValueError(f"Gene {gid} has mismatched RNA transcripts: {tids} vs {self.genes[gid].transcripts}")


def read_pkl[GeneAttrs: Mapping, GeneTag: Hashable, RNAAttrs: Mapping, RNATag: Hashable](
        path: str
) -> Annotome[GeneAttrs, GeneTag, RNAAttrs, RNATag]:
    with open(path, "rb") as f:
        return pickle.load(f)
