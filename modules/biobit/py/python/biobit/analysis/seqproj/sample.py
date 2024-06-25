from attrs import define, field


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class Sample:
    """
    A class to represent a sample in a biological experiment.

    Attributes
    ----------
    ind : str
        Sample ID, must be unique within the project.
    organism : set[str]
        Organism(s) from which the sample was derived. E.g. {'Homo sapiens'}, {'Mus musculus', 'HSV-1'}, etc.
    attributes: dict[str, str]
        Additional descriptive attributes for the sample, optional. E.g. {'Confluence': '75%', 'Source': 'HeLa'}, etc.
    description: str
        A description of the sample, if available.
    """
    ind: str = field()
    organism: set[str] = field(converter=lambda x: set(x))
    # Optional meta information
    attributes: dict[str, str] = field(factory=dict)
    description: str | None = field(default=None)

    @ind.validator
    def check_ind(self, _, value):
        if not value:
            raise ValueError("Sample ID must be specified")

    @organism.validator
    def check_organism(self, _, value):
        if not value:
            raise ValueError("Organism must be specified")

    @attributes.validator
    def check_tags(self, _, value):
        for k, v in value.items():
            if not v:
                raise ValueError(f"Empty attributes are not allowed: {k} -> {v}")

    # def __repr__(self) -> str:
    #     return f"Sample({self.ind}, {self.organism}, {self.attributes}, {self.description})"
    #
    # def __str__(self) -> str:
    #     return (f"Sample({self.ind}):\n"
    #             f"\tOrganism: {', '.join(self.organism)}\n"
    #             f"\tAttributes: {', '.join(f'{k}={v}' for k, v in self.attributes.items())}\n"
    #             f"\tDescription: {self.description or '.'}")
