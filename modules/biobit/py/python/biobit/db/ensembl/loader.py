import json
from enum import EnumMeta, Enum
from pathlib import Path
from typing import Self

from biobit.paths import CACHE
from . import gene, transcript
from .assembly import Assembly

ENSEMBL_RELEASES = {
    107: "http://jul2022.archive.ensembl.org",
    108: "http://oct2022.archive.ensembl.org",
    109: "http://feb2023.archive.ensembl.org",
    110: "http://jul2023.archive.ensembl.org",
    111: "http://jan2024.archive.ensembl.org",
    112: "http://may2024.archive.ensembl.org",
}
LATEST = 112


class Loader:
    def __init__(
            self,
            name: str,
            organism: str,
            version: int,
            cache: Path | None = None,
    ):
        self._assembly = name
        self._organism = organism
        if version not in ENSEMBL_RELEASES:
            raise ValueError(
                f"Unknown Ensembl release {version} version, known releases are: {ENSEMBL_RELEASES.keys()}"
            )
        self._version = version
        self._url = ENSEMBL_RELEASES[version]
        self._cache = (
            CACHE / "ensembl" / self._organism / self._assembly / str(version)
            if cache is None
            else cache
        )
        self._cache.mkdir(parents=True, exist_ok=True)

        # Load meta information
        self._meta_path = self._cache / "meta-information.json"
        if self._meta_path.exists():
            with open(self._meta_path) as f:
                self._meta = json.load(f)
            if self._meta["schema"] != "0.1":
                raise ValueError(f"Unknown schema version {self._meta['schema']}")

            if (
                    self._meta["organism"] != organism
                    or self._meta["name"] != name
                    or self._meta["version"] != version
            ):
                raise ValueError(
                    f"Meta information mismatch: {self._meta} != {organism}, {name}, {version}. "
                    f"Please delete the cache folder ({self._cache}) and try again."
                )
        else:
            self._meta = {
                "schema": "0.1",
                "organism": organism,
                "name": name,
                "version": version,
                "cached": {
                    "transcript_attributes": [],
                    "gene_attributes": [],
                },
            }

        self.transcripts: transcript.Descriptor | None = None
        self.genes: gene.Descriptor | None = None

    def auto(self, fetch: bool = False, verbose: bool = True) -> Assembly:
        if self._version != LATEST:
            raise ValueError(
                f"Auto is supported only for the latest (version {LATEST}) GRCh38 (hsapiens) and "
                f"GRCm39 (mmusculus) Ensembl assemblies"
            )

        match (self._assembly, self._organism):
            case ("GRCh38", "hsapiens"):
                self.with_transcripts(
                    set(transcript.Attribute), fetch=fetch, verbose=verbose
                ).with_genes(set(gene.Attribute), fetch=fetch, verbose=verbose)
            case ("GRCm39", "mmusculus"):
                attributes = set(transcript.Attribute) - {
                    transcript.Attribute.MANEPlusClinical,
                    transcript.Attribute.MANESelect,
                }
                self.with_transcripts(
                    attributes, fetch=fetch, verbose=verbose
                ).with_genes(set(gene.Attribute), fetch=fetch, verbose=verbose)
            case _:
                raise ValueError(
                    f"Auto is supported only for the latest (version {LATEST}) GRCh38 (hsapiens) and "
                    f"GRCm39 (mmusculus) Ensembl assemblies"
                )
        return self.finalize()

    def with_transcripts(
            self,
            attributes: set[transcript.Attribute],
            fetch: bool = False,
            verbose: bool = True,
    ) -> Self:
        path = self._cache / "transcript-attributes.tsv.gz"
        self._load_attributes(
            path,
            "transcript_attributes",
            transcript.Attribute,
            attributes,
            fetch,
            verbose,
        )
        self.transcripts = transcript.Descriptor(attributes, path)
        return self

    def with_genes(
            self, attributes: set[gene.Attribute], fetch: bool = False, verbose: bool = True
    ) -> Self:
        path = self._cache / "gene-attributes.tsv.gz"
        self._load_attributes(
            path, "gene_attributes", gene.Attribute, attributes, fetch, verbose
        )
        self.genes = gene.Descriptor(attributes, path)
        return self

    def finalize(self) -> Assembly:
        return Assembly(
            self._assembly, self._organism, self._version, self.transcripts, self.genes
        )

    def _load_attributes(
            self,
            path: Path,
            key: str,
            enum: EnumMeta,
            attributes: set,
            fetch: bool,
            verbose: bool,
    ):
        cached: set[Enum] = {enum[x] for x in self._meta["cached"][key]}

        # Check if we need to fetch the data
        if attributes.issubset(cached):
            return

        if not fetch:
            raise ValueError(
                f"Attributes {attributes} for {key} are not available locally, please set fetch=True to download them"
            )

        # Fetch the data
        all_attributes = cached | attributes
        enum.fetch(  # type: ignore
            all_attributes,
            self._organism,
            path,
            force=True,
            url=self._url,
            verbose=verbose,
        )
        self._meta["cached"][key] = [x.name for x in all_attributes]
        self._update_meta()

    def _update_meta(self):
        with open(self._meta_path, "w") as f:
            json.dump(self._meta, f)
