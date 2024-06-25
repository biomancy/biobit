from pathlib import Path

import cattrs.preconf.pyyaml

from ...seqrun import SeqRun, SeqLayout


def register_hooks(converter: cattrs.Converter) -> cattrs.Converter:
    def unstructure(run: SeqRun) -> dict:
        dictionary = {
            "ind": run.ind,
            "machine": run.machine,
            "layout": converter.unstructure(run.layout),
            "files": converter.unstructure(run.files),
        }

        if run.reads:
            dictionary["reads"] = run.reads
        if run.bases:
            dictionary["bases"] = run.bases
        if run.description:
            dictionary["description"] = run.description
        return dictionary

    def structure(data: dict, ttype: type) -> SeqRun:
        assert ttype is SeqRun

        ind = converter.structure(data["ind"], str)
        machine = converter.structure(data["machine"], str)
        layout = converter.structure(data["layout"], SeqLayout)
        files = converter.structure(data["files"], tuple[Path, ...])

        reads = converter.structure(data["reads"], int) if "reads" in data else None
        bases = converter.structure(data["bases"], int) if "bases" in data else None
        description = converter.structure(data["description"], str) if "description" in data else None
        return SeqRun(ind, machine, layout, files, reads, bases, description)

    converter.register_unstructure_hook(SeqRun, unstructure)
    converter.register_structure_hook(SeqRun, structure)
    return converter
