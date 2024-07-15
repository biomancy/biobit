from attrs import define, field, converters

from biobit.core.ngs import Strandedness


@define(slots=True, frozen=True, eq=True, order=True, hash=True, repr=True, str=True)
class Library:
    """
    A class to describe meta information about a sequencing library made from a biological sample.

    Attributes
    ----------
    source : set[str]
        What molecules were used to generate the library? E.g. DNA, RNA, etc.
    selection : set[str]
        Were there any selection/enrichment steps during library generation? E.g. Poly-A, Ribo-Zero, RIP, etc.
    strandedness : Strandedness
        Indicates the relationship between molecules in the library and their source DNA/RNA strand. When not available,
        should be set to None.
    attributes : dict[str, str]
        Additional descriptive attributes for the library, optional. E.g. {'Kit ID': '106-301'}, etc.
    """
    source: set[str] = field(converter=lambda x: set(x))
    selection: set[str] = field(converter=lambda x: set(x))
    strandedness: Strandedness | None = field(default=None, converter=converters.optional(lambda x: Strandedness(x)))
    attributes: dict[str, str] = field(factory=dict)

    @source.validator
    def _check_source(self, _, value):
        if not value:
            raise ValueError("Library source must be specified")

    @selection.validator
    def _check_selection(self, _, value):
        if not value:
            raise ValueError("Library selection method must be specified")
