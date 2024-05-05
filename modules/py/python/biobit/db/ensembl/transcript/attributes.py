from pathlib import Path
from typing import Any, Self

from biobit.db.ensembl.abc import EnsemblAttribute
from .. import biomart


class Attribute(EnsemblAttribute):
    ID = "ID"
    Gene = "gene"
    Name = "name"
    Biotype = "biotype"
    Contig = "contig"
    Start = "start"
    End = "end"
    Strand = "strand"
    Source = "source"
    SupportLevel = "support level"
    MANESelect = "MANE Select"
    MANEPlusClinical = "MANE Plus Clinical"
    GENCODEbasic = "GENCODE basic"
    EnsemblCanonical = "Ensembl Canonical"

    @property
    def colname(self) -> str:
        match self:
            case Attribute.ID:
                return "Transcript stable ID version"
            case Attribute.Gene:
                return "Gene stable ID version"
            case Attribute.Name:
                return "Transcript name"
            case Attribute.Biotype:
                return "Transcript type"
            case Attribute.SupportLevel:
                return "Transcript support level (TSL)"
            case Attribute.Contig:
                return "Chromosome/scaffold name"
            case Attribute.Start:
                return "Transcript start (bp)"
            case Attribute.End:
                return "Transcript end (bp)"
            case Attribute.Strand:
                return "Strand"
            case Attribute.MANESelect:
                return "RefSeq match transcript (MANE Select)"
            case Attribute.MANEPlusClinical:
                return "RefSeq match transcript (MANE Plus Clinical)"
            case Attribute.GENCODEbasic:
                return "GENCODE basic annotation"
            case Attribute.EnsemblCanonical:
                return "Ensembl Canonical"
            case Attribute.Source:
                return "Source (transcript)"
            case _:
                raise ValueError(f"Unknown attribute {self}")

    @property
    def dtype(self) -> Any:
        match self:
            case (
            Attribute.ID
            | Attribute.Gene
            | Attribute.Name
            | Attribute.Biotype
            | Attribute.Source
            | Attribute.SupportLevel
            ):
                return str
            case (
            Attribute.MANESelect
            | Attribute.MANEPlusClinical
            | Attribute.GENCODEbasic
            ):
                return str
            case Attribute.EnsemblCanonical:
                return "boolean"
            case Attribute.Contig:
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
                    requested.append("ensembl_transcript_id_version")
                case Attribute.Gene:
                    requested.append("ensembl_gene_id_version")
                case Attribute.Name:
                    requested.append("external_transcript_name")
                case Attribute.Biotype:
                    requested.append("transcript_biotype")
                case Attribute.SupportLevel:
                    requested.append("transcript_tsl")
                case Attribute.Contig:
                    requested.append("chromosome_name")
                case Attribute.Start:
                    requested.append("transcript_start")
                case Attribute.End:
                    requested.append("transcript_end")
                case Attribute.Strand:
                    requested.append("strand")
                case Attribute.MANESelect:
                    requested.append("transcript_mane_select")
                case Attribute.MANEPlusClinical:
                    requested.append("transcript_mane_plus_clinical")
                case Attribute.GENCODEbasic:
                    requested.append("transcript_gencode_basic")
                case Attribute.EnsemblCanonical:
                    requested.append("transcript_is_canonical")
                case Attribute.Source:
                    requested.append("transcript_source")
                case _:
                    raise ValueError(f"Unknown attribute {attr}")

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
