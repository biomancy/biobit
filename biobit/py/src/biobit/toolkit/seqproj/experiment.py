from attrs import define, field

from biobit.core import ngs
from .library import Library
from .run import Run, Layout
from .sample import Sample


# TODO: make sure that libraries are shareable between experiments + add IND to the library
# TODO: Should I enforce the fact that runs are homogeneous? That would make the NGS layout easier to deduce / store.
# But I would loose the flexibility of having different runs for the same experiment. That's not quite common anyway
# and, perhaps, I can circumvent it downstream by allowing people to sum up experiments where needed.
# Or not really?

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
    runs: tuple[Run, ...] = field(converter=lambda x: tuple(x))
    # Optional fields
    attributes: dict[str, str] = field(factory=dict)
    description: str | None = field(default=None)

    def ngs(self) -> ngs.Layout:
        """
        Return the NGS layout of the experiment, derived from the library layout and the runs. Throws an error if the
        experiment is heterogeneous and has more than one sequencing layout.
        """
        strandedness = self.library.strandedness
        if strandedness is None:
            raise ValueError("Experiment has library with unknown strandedness, can't deduce NGS layout.")

        paired, single = set(), 0
        for run in self.runs:
            if isinstance(run.layout, Layout.Single):
                single += 1
            elif isinstance(run.layout, Layout.Paired):
                paired.add(run.layout.orientation)

        if paired and single > 0:
            raise ValueError("Experiment has mixed run layouts, can't deduce NGS layout.")
        elif paired:
            if len(paired) > 1:
                raise ValueError("Experiment has multiple paired-end layouts, can't deduce NGS layout.")
            orientation = paired.pop()
            if not orientation:
                raise ValueError("Experiment has paired-end layout with unknown orientation, can't deduce NGS layout.")
            return ngs.Layout.Paired(strandedness, orientation)
        elif single:
            return ngs.Layout.Single(strandedness)
        else:
            raise ValueError("Experiment has no runs, can't deduce NGS layout.")

    @ind.validator
    def check_ind(self, _, value):
        if not value:
            raise ValueError("Experiment ID must be specified")

    @runs.validator
    def check_runs(self, _, value):
        if not value:
            raise ValueError("At least one sequencing run must be specified")

        # Ensure that run IDs are unique within the experiment
        cnts: dict[str, int] = {}
        for x in value:
            cnts[x.ind] = cnts.get(x.ind, 0) + 1
        nonunique = {k for k, v in cnts.items() if v > 1}
        if nonunique:
            raise ValueError(f"Run IDs must be unique within the experiment, found duplicates: {nonunique}")

    @description.validator
    def check_description(self, _, value):
        if value is not None and not value:
            raise ValueError("If specified, description must be non-empty. Use None to indicate lack of description")
