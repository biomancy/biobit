from collections import Counter

import cattrs.preconf.pyyaml

from ...experiment import Experiment
from ...library import Library
from ...project import Project
from ...sample import Sample
from ...seqrun import SeqRun


def register_hooks(converter: cattrs.Converter) -> cattrs.Converter:
    def unstructure(exp: Experiment) -> dict:
        dictionary = {
            "ind": exp.ind,
            "sample": exp.sample.ind,
            "library": converter.unstructure(exp.library),
            "runs": converter.unstructure(exp.runs)
        }
        if exp.attributes:
            dictionary["attributes"] = exp.attributes
        if exp.description:
            dictionary["description"] = exp.description
        return dictionary

    def structure(data: dict, ttype: type) -> Project:
        assert ttype is Project

        samples = converter.structure(data["samples"], tuple[Sample, ...])
        non_unique = [(k, v) for k, v in Counter(s.ind for s in samples).items() if v >= 2]
        if non_unique:
            raise ValueError(f"Sample IDs must be unique, got: {non_unique}")

        samples_mapping = {s.ind: s for s in samples}
        experiments = tuple(Experiment(
            data["ind"],
            samples_mapping[data["sample"]],
            converter.structure(data["library"], Library),
            converter.structure(data["runs"], tuple[SeqRun, ...]),
            converter.structure(data["attributes"], dict[str, str]) if "attributes" in data else {},
            converter.structure(data["description"], str) if "description" in data else None,
        ) for data in data["experiments"])

        return Project(data["ind"], tuple(experiments), samples)

    converter.register_unstructure_hook(Experiment, unstructure)
    converter.register_structure_hook(Project, structure)
    return converter
