from attrs import define, field, converters

from .layout import Layout


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class Run:
    """
    A class to represent a single run in a sequencing experiment.

    Attributes
    ----------
    ind : str
        Index of the sequencing run, should be unique within the project.
    machine : str
        Sequencing machine used for the run. E.g. 'Illumina NovaSeq 6000', 'Oxford Nanopore MinION', etc.
    layout : Layout
        Layout of the sequencing run and paths to associated sequencing files (e.g. FASTQ files) with associated
        file paths.
    reads : int
        Total number of reads in the sequencing run, if available.
    bases : int
        Total number of bases in the sequencing run, if available.
    description : str
        Description of the sequencing run, if available.
    """
    ind: str = field()
    layout: Layout = field()
    # Optional metadata
    machine: str | None = field(default=None)
    reads: int | None = field(default=None, converter=converters.optional(lambda x: int(x)))
    bases: int | None = field(default=None, converter=converters.optional(lambda x: int(x)))
    description: str | None = field(default=None)

    @ind.validator
    def check_ind(self, _, value):
        if not value:
            raise ValueError("Sequencing run index must be specified")

    @machine.validator
    def check_machine(self, _, value):
        if isinstance(value, str) and not value:
            raise ValueError("Use None instead of an empty string to indicate lack of machine information")

    @reads.validator
    def check_reads(self, _, value):
        if value is not None and (value <= 0 or not isinstance(value, int)):
            raise ValueError("Total number of reads must be a positive integer")

    @bases.validator
    def check_bases(self, _, value):
        if value is not None and (value <= 0 or not isinstance(value, int)):
            raise ValueError("Total number of bases must be a positive integer")

    @description.validator
    def check_description(self, _, value):
        if value is not None and not value:
            raise ValueError("If specified, description must be non-empty. Use None to indicate lack of description")

    # def __repr__(self) -> str:
    #     files = ", ".join(map(str, self.files))
    #     return f"SeqRun({self.ind}, {self.machine}, {self.layout}, ({files}), {self.reads}, {self.bases}, {self.description})"
    #
    # def __str__(self) -> str:
    #     fields = [
    #         f"\tMachine: {self.machine}",
    #         f"\tLayout: {self.layout}",
    #         f"\tFiles: {', '.join(map(str, self.files))}",
    #         f"\tReads: {self.reads if self.reads else '.'}",
    #         f"\tBases: {self.bases if self.bases else '.'}",
    #         f"\tDescription: {self.description if self.description else '.'}"
    #     ]
    #     body = "\n".join(fields)
    #     return f"SeqRun({self.ind}):\n{body}"
