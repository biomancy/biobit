from collections.abc import Iterable
from typing import Callable

from attrs import define, field

from .experiment import Experiment
from .library import Library
from .project import Project
from .sample import Sample
from .run import Run, Layout


@define(slots=True)
class FilteringSchema:
    _project_filters: list[Callable[[Project], bool]] = field(factory=list, init=False)
    _experiment_filters: list[Callable[[Project, Experiment], bool]] = field(factory=list, init=False)
    _sample_filters: list[Callable[[Project, Sample], bool]] = field(factory=list, init=False)
    _run_filters: list[Callable[[Project, Experiment, Run], bool]] = field(factory=list, init=False)
    _library_filters: list[Callable[[Project, Experiment, Library], bool]] = field(factory=list, init=False)

    def with_project_filters(self, *fns: Callable[[Project], bool]) -> "FilteringSchema":
        self._project_filters.extend(fns)
        return self

    def with_experiment_filters(self, *fns: Callable[[Project, Experiment], bool]) -> "FilteringSchema":
        self._experiment_filters.extend(fns)
        return self

    def with_sample_filters(self, *fns: Callable[[Project, Sample], bool]) -> "FilteringSchema":
        self._sample_filters.extend(fns)
        return self

    def with_run_filters(self, *fns: Callable[[Project, Experiment, Run], bool]) -> "FilteringSchema":
        self._run_filters.extend(fns)
        return self

    def with_library_filters(self, *fns: Callable[[Project, Experiment, Library], bool]) -> "FilteringSchema":
        self._library_filters.extend(fns)
        return self

    def with_project_ind(self, allowed: Iterable[str] | str) -> "FilteringSchema":
        if isinstance(allowed, str):
            options = {allowed}
        else:
            options = set(allowed)
        return self.with_project_filters(lambda prj: prj.ind in options)

    def with_attribute(self, attribute: str, allowed: Iterable[str] | str) -> "FilteringSchema":
        if isinstance(allowed, str):
            options = {allowed}
        else:
            options = set(allowed)
        return self.with_sample_filters(lambda _, sample: sample.attributes.get(attribute) in options)

    def with_seqlayout(self, allowed: Iterable[Layout] | Layout) -> "FilteringSchema":
        if isinstance(allowed, Layout):
            options = {allowed}
        else:
            options = set(allowed)
        return self.with_run_filters(lambda _, __, run: run.layout in options)

    def __call__(self, projects: Iterable[Project]) -> Iterable[Project]:
        for prj in projects:
            if not all(fn(prj) for fn in self._project_filters):
                continue

            # Select only samples that passed the filters
            samples = {sample.ind for sample in prj.samples if all(fn(prj, sample) for fn in self._sample_filters)}
            if not samples:
                continue

            # Select only experiments that passed the filters
            passed_experiments = []
            for exp in prj.experiments:
                # Check if the sample passed the filters
                if exp.sample.ind not in samples:
                    continue
                # Check if library pass the filters
                if not all(fn(prj, exp, exp.library) for fn in self._library_filters):
                    continue
                # Check if at least one run pass the filters
                runs = tuple(run for run in exp.runs if all(fn(prj, exp, run) for fn in self._run_filters))
                if not runs:
                    continue
                # Add the experiment to the list of passed experiments
                passed_experiments.append(
                    Experiment(exp.ind, exp.sample, exp.library, runs, exp.attributes, exp.description)
                )

            # Keep only samples that are present in at least one experiment
            samples = {exp.sample.ind for exp in passed_experiments}

            if passed_experiments:
                yield Project(
                    prj.ind,
                    experiments=tuple(passed_experiments),
                    samples=tuple(sample for sample in prj.samples if sample.ind in samples),
                    description=prj.description
                )
