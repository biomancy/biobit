from biobit.core.loc import Orientation

from .pileup import TaskPileup

class SamplePileup[T]:
    """
    REAT output for one sample tag.

    Pileups are grouped by exact `(seqid, orientation)` keys. Missing keys mean no sites were
    selected for that sequence/orientation. `pileups()` builds and returns a new dictionary each
    time; mutating it does not mutate the underlying result object.
    """

    tag: T

    def pileups(self) -> dict[tuple[str, Orientation], TaskPileup]:
        """Return task pileups grouped by `(seqid, orientation)`."""
        ...

    def len(self) -> int: ...
    def is_empty(self) -> bool: ...
