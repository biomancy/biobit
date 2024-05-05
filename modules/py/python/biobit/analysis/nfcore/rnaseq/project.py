import os
from pathlib import Path
from typing import Callable

from attrs import define, field

from biobit.analysis import seqproj
from . import descriptor
from .experiment import Experiment


@define(slots=True, frozen=True, eq=True, order=True, repr=True, hash=True, init=False)
class Project:
    source: seqproj.Project = field()
    experiments: tuple[Experiment, ...] = field(converter=lambda x: tuple(x))

    def __init__(
            self, source: seqproj.Project, resfolder: os.PathLike[str], *,
            bioexp2descriptor: Callable[[seqproj.Experiment], str] = descriptor.from_bioexp,
            ensure_exists: bool = False
    ):
        resfolder = Path(resfolder)
        # if not resfolder.is_dir():
        #     raise ValueError(f"Results folder {resfolder} does not exist")

        experiments = []
        for exp in source.experiments:
            descriptor = bioexp2descriptor(exp)
            experiments.append(Experiment(descriptor, exp, resfolder))

            if ensure_exists:
                experiments[-1].ensure_exists()

        self.__attrs_init__(source, experiments)

    @experiments.validator
    def check_experiments(self, _, value):
        if not value:
            raise ValueError("Project must contain at least one experiment")

    @property
    def ind(self) -> str:
        return self.source.ind

    def ensure_exists(self):
        for exp in self.experiments:
            exp.ensure_exists()
