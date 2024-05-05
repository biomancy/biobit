from pathlib import Path
from typing import Self

from .. import biomart
from ..abc import EnsemblAttribute


class Attribute(EnsemblAttribute):
    ID = "ID"
    Name = "name"
    Biotype = "biotype"
    Source = "source"
    Contig = "contig"
    Start = "start"
    End = "end"
    Strand = "strand"

    @property
    def colname(self) -> str:
        match self:
            case Attribute.ID:
                return "Gene stable ID version"
            case Attribute.Name:
                return "Gene name"
            case Attribute.Biotype:
                return "Gene type"
            case Attribute.Source:
                return "Source (gene)"
            case Attribute.Contig:
                return "Chromosome/scaffold name"
            case Attribute.Start:
                return "Gene start (bp)"
            case Attribute.End:
                return "Gene end (bp)"
            case Attribute.Strand:
                return "Strand"
            case _:
                raise ValueError(f"Unknown attribute {self}")

    @property
    def dtype(self):
        match self:
            case (
            Attribute.ID
            | Attribute.Name
            | Attribute.Biotype
            | Attribute.Source
            | Attribute.Contig
            ):
                return str
            case Attribute.Start | Attribute.End | Attribute.Strand:
                return int
            case _:
                raise ValueError(f"Unknown attribute {self}")

    @classmethod
    def fetch(
            cls,
            attributes: set[Self],
            organism: str,
            saveto: Path,
            force: bool,
            url: str,
            verbose: bool,
    ):
        requested = []
        for attr in attributes:
            match attr:
                case Attribute.ID:
                    requested.append("ensembl_gene_id_version")
                case Attribute.Name:
                    requested.append("external_gene_name")
                    # requested.append("external_gene_source")
                    # requested.append("external_synonym")
                case Attribute.Biotype:
                    requested.append("gene_biotype")
                case Attribute.Source:
                    requested.append("source")
                case Attribute.Contig:
                    requested.append("chromosome_name")
                case Attribute.Start:
                    requested.append("start_position")
                case Attribute.End:
                    requested.append("end_position")
                case Attribute.Strand:
                    requested.append("strand")
                case _:
                    raise ValueError(f"Unknown attribute: {attr}")

        requested = [f'<Attribute name = "{x}" />' for x in requested]
        body = "\n".join(requested)
        query = f"""
            <?xml version="1.0" encoding="UTF-8"?>
            <!DOCTYPE Query>
            <Query  virtualSchemaName = "default" formatter = "TSV" header = "1" uniqueRows = "1" count = "" datasetConfigVersion = "0.6" >
                <Dataset name = "{organism}_gene_ensembl" interface = "default" >
                    {body}
                </Dataset>
            </Query>
        """

        biomart.fetch(query, url, saveto, verbose=verbose, force=force)
