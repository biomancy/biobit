from collections import defaultdict
from collections.abc import Callable

from attrs import define, field

from .experiment import Experiment
from .sample import Sample
from .seqrun import SeqRun


@define(hash=True, slots=True, frozen=True, eq=True, order=True, repr=True, str=True)
class Project:
    """
    A class representing a biological project, which consists of one or more biological experiments.

    Attributes
    ----------
    ind : str
        Project ID, must be globally unique.
    experiments : tuple[Experiment, ...]
        The experiments in the project.
    samples : tuple[Sample, ...]
        The samples in the project.
    description : str
        A description of the project, if available.
    """
    ind: str = field()
    experiments: tuple[Experiment, ...] = field(converter=lambda x: tuple(x))
    samples: tuple[Sample, ...] = field(converter=lambda x: tuple(x))
    # Optional fields
    description: str | None = field(default=None)

    @ind.validator
    def check_ind(self, _, value):
        if not value:
            raise ValueError("Project ID must be specified")

    @experiments.validator
    def check_experiments(self, _, value):
        if not value:
            raise ValueError("At least one experiment must be specified")

    @samples.validator
    def check_samples(self, _, value):
        if not value:
            raise ValueError("At least one sample must be specified")

    @description.validator
    def check_description(self, _, value):
        if value is not None and not value:
            raise ValueError("If specified, description must be non-empty. Use None to indicate lack of description")

    def __attrs_post_init__(self):
        # Ensure that experiment IDs are unique within the project
        cnts = {}
        for x in self.experiments:
            cnts[x.ind] = cnts.get(x.ind, 0) + 1
        nonunique = {k for k, v in cnts.items() if v > 1}
        if nonunique:
            raise ValueError(f"Experiment IDs must be unique within the project, found duplicates: {nonunique}")

        # Ensure that run IDs are unique within the project
        cnts = {}
        for x in self.experiments:
            for y in x.runs:
                cnts[y.ind] = cnts.get(y.ind, 0) + 1
        nonunique = {k for k, v in cnts.items() if v > 1}
        if nonunique:
            raise ValueError(f"Run IDs must be unique within the project, found duplicates: {nonunique}")

        # Gather all samples from the experiments
        samples = defaultdict(list)
        for x in self.experiments:
            samples[x.sample.ind].append(x.sample)

        # Ensure that samples with the same ID are identical
        for k, v in samples.items():
            if len(v) > 1 and any(x != v[0] for x in v[1:]):
                raise ValueError(f"Samples with the same ID must be identical, found differences for sample {k}: {v}")

    def subsample(
            self,
            newind: str | None = None,
            experiments: Callable[[Experiment], bool] | None = None,
            samples: Callable[[Experiment, Sample], bool] | None = None,
            runs: Callable[[Experiment, SeqRun], bool] | None = None
    ) -> "Project":
        """
        Subsample the project based on the specified criteria.
        :param newind: new project ID. If None, the original ID is used.
        :param experiments: callable that takes an experiment and returns True if it should be included, False
            otherwise. If None, all experiments are included.
        :param samples: callable that takes an experiment and a sample and returns True if the sample should be
            included, False otherwise. If None, all samples are included.
        :param runs: callable that takes an experiment and a sequencing run and returns True if the run should be
            included, False otherwise. If None, all runs are included.
        :return: a new Project instance including only the specified experiments, samples, and runs
        """
        raise ValueError("Not implemented")
