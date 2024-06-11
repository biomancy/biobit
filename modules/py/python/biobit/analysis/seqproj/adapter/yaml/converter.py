import os
from io import TextIOBase

import cattrs.preconf.pyyaml

from . import experiment, library, sample, seqrun
from ...project import Project


def build():
    converter = cattrs.preconf.pyyaml.make_converter()

    library.register_hooks(converter)
    seqrun.register_hooks(converter)
    sample.register_hooks(converter)
    experiment.register_hooks(converter)

    return converter


_YAML_CONVERTER = build()


def load(file: os.PathLike[str] | TextIOBase) -> Project:
    if isinstance(file, TextIOBase):
        return _YAML_CONVERTER.loads(file.read(), Project)
    else:
        with open(file) as f:
            return _YAML_CONVERTER.loads(f.read(), Project)


def loads(string: str) -> Project:
    return _YAML_CONVERTER.loads(string, Project)


def dump(project: Project, saveto: os.PathLike[str] | TextIOBase) -> str:
    string = dumps(project)
    if isinstance(saveto, TextIOBase):
        saveto.write(string)
    else:
        with open(saveto, "w") as f:
            f.write(string)
    return string


def dumps(project: Project) -> str:
    return _YAML_CONVERTER.dumps(project, sort_keys=False)
