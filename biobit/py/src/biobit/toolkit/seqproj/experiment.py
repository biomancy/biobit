from attrs import define, field

from .library import Library
from .sample import Sample
from .seqrun import SeqRun


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class Experiment:
    """
    A class representing a sequencing experiment, which consists of a biological material isolated from a sample,
    turned into a sequencing library, and sequenced one or more times.

    Attributes
    ----------
    ind : str
        Experiment ID, must be unique within the project.
    sample : Sample
        The biological sample from which the library was generated.
    library : Library
        The library generated from the sample.
    runs : tuple[SeqRun, ...]
        The sequencing runs performed on the library.
    attributes : dict[str, str]
        Additional descriptive attributes for the library, optional. E.g. {'title': 'Hela-rep-2'}, etc.
    description : str
        A description of the experiment, if available.
    """
    ind: str = field()
    sample: Sample = field()
    library: Library = field()
    runs: tuple[SeqRun, ...] = field(converter=lambda x: tuple(x))
    # Optional fields
    attributes: dict[str, str] = field(factory=dict)
    description: str | None = field(default=None)

    @ind.validator
    def check_ind(self, _, value):
        if not value:
            raise ValueError("Experiment ID must be specified")

    @runs.validator
    def check_runs(self, _, value):
        if not value:
            raise ValueError("At least one sequencing run must be specified")

        # Ensure that run IDs are unique within the experiment
        cnts = {}
        for x in value:
            cnts[x.ind] = cnts.get(x.ind, 0) + 1
        nonunique = {k for k, v in cnts.items() if v > 1}
        if nonunique:
            raise ValueError(f"Run IDs must be unique within the experiment, found duplicates: {nonunique}")

    @description.validator
    def check_description(self, _, value):
        if value is not None and not value:
            raise ValueError("If specified, description must be non-empty. Use None to indicate lack of description")
