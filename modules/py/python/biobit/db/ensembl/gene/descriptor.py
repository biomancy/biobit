from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import pandas as pd

from .attributes import Attribute
from ..abc import EnsemblDescriptor


@dataclass(frozen=True, slots=True)
class Record:
    id: str
    # Attributes
    name: str | None = None
    biotype: str | None = None
    source: str | None = None
    # Coordinates
    contig: str | None = None
    start: int | None = None
    end: int | None = None
    strand: Literal["+", "-"] | None = None

    @property
    def coordinates(self) -> str | None:
        if (
                self.contig is None
                or self.start is None
                or self.end is None
                or self.strand is None
        ):
            return None
        return f"{self.contig}:{self.start}-{self.end}({self.strand})"


class Descriptor(EnsemblDescriptor):
    def __init__(self, attributes: set[Attribute], cache: Path):
        self.attributes = attributes
        self.cache = cache
        self._records: dict[str, Record] = {}

        if attributes and Attribute.ID not in attributes:
            raise ValueError(
                f"Attribute {Attribute.ID} must be included when querying transcript attributes"
            )

        dtypes = {x.colname: x.dtype for x in attributes}
        df = pd.read_csv(
            self.cache, sep="\t", dtype=dtypes, usecols=list(dtypes.keys())
        )

        if Attribute.Strand in attributes:
            assert set(df[Attribute.Strand.colname].unique()) == {1, -1}, df[
                Attribute.Strand.colname
            ].unique()
            df[Attribute.Strand.colname] = df[Attribute.Strand.colname].apply(
                lambda x: "+" if x > 0 else "-"
            )

        # Create records
        records = [Record(id=id) for id in df[Attribute.ID.colname]]
        mapping = {
            Attribute.Name: "name",
            Attribute.Biotype: "biotype",
            Attribute.Source: "source",
            Attribute.Contig: "contig",
            Attribute.Start: "start",
            Attribute.End: "end",
            Attribute.Strand: "strand",
        }
        for attr in attributes - {Attribute.ID}:
            for r, val in zip(records, df[attr.colname]):
                val = val if not pd.isna(val) else None
                object.__setattr__(r, mapping[attr], val)

        for r in records:
            self._records[r.id] = r
            self._records[r.id.split(".")[0]] = r

    def record(self, gid: str) -> Record | None:
        return self._records.get(gid, None)

    # # Gene id -> gene coordinates
    # self._id2coords = None
    # if all(attr in attributes for attr in [Attribute.Contig, Attribute.Start, Attribute.End, Attribute.Strand]):
    #     self._id2coords = {}
    #     cols = [Attribute.Contig.colname, Attribute.Start.colname, Attribute.End.colname, Attribute.Strand.colname]
    #     for id in idcols:
    #         self._id2coords |= dict(zip(df[id], df[cols].itertuples(index=False, name=None)))
    #
    # # Gene id -> gene biotype
    # self._id2biotype = None
    # if Attribute.Biotype in attributes:
    #     self._id2biotype = {}
    #     for id in idcols:
    #         self._id2biotype |= dict(zip(df[id], df[Attribute.Biotype.colname]))
    #
    # # Gene id -> gene name; Gene name -> gene id
    # self._id2name, self._name2id = None, None
    # if Attribute.Name in attributes:
    #     self._id2name = {}
    #     for id in idcols:
    #         self._id2name |= dict(zip(df[id], df[Attribute.Name.colname]))
    #     self._name2id = dict(zip(df[Attribute.Name.colname], df[Attribute.ID.colname]))

    # Synonyms -> gene id
    #     self._synonym2id = self._name2id.copy()
    #     self._synonym2id.update(
    #         {k: v for k, v in zip(df['Gene Synonym'].values, df['Gene stable ID version'].values) if not pd.isna(k)}
    #     )
    #     # self._synonym2id.update({
    #     #     k: v
    #     #     for k, v in zip(df['NCBI gene (formerly Entrezgene) accession'].values, df['Gene stable ID version'].values)
    #     # })
    #     self._synonym2id.update({k.lower(): v for k, v in self._synonym2id.items()})
    #     self._synonym2id.update({k.upper(): v for k, v in self._synonym2id.items()})
    #
    # def id2coords(self, id: str) -> Optional[Tuple[str, int, int, str]]:
    #     return self._getter(self._id2coords, id)
    #
    # def id2biotype(self, id: str) -> Optional[str]:
    #     return self._getter(self._id2biotype, id)
    #
    # def id2name(self, id: str) -> Optional[str]:
    #     return self._getter(self._id2name, id)
    #
    # def id2transcripts(self, id: str) -> Optional[Tuple[str, ...]]:
    #     return self._getter(self._id2transcripts, id)
    #
    # def name2id(self, name: str) -> Optional[str]:
    #     return self._getter(self._name2id, name)
    #
    # def synonym2id(self, synonym: str) -> Optional[str]:
    #     gid = self._getter(self._synonym2id, synonym)
    #     if gid is None:
    #         gid = self._getter(self._synonym2id, synonym.lower())
    #     if gid is None:
    #         gid = self._getter(self._synonym2id, synonym.upper())
    #     return gid
