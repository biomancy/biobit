from . import gene, transcript


class Assembly:
    def __init__(
            self,
            name: str,
            organism: str,
            version: int,
            transcripts: transcript.Descriptor | None = None,
            genes: gene.Descriptor | None = None,
    ):
        self._name = name
        self._organism = organism
        self._version = version

        self._transcripts = transcripts
        self._genes = genes

    @property
    def name(self) -> str:
        return self._name

    @property
    def organism(self) -> str:
        return self._organism

    @property
    def version(self) -> int:
        return self._version

    @property
    def transcripts(self) -> transcript.Descriptor:
        if self._transcripts is None:
            raise ValueError("Transcripts information is not available")
        return self._transcripts

    @property
    def genes(self) -> gene.Descriptor:
        if self._genes is None:
            raise ValueError("Genes information is not available")
        return self._genes
