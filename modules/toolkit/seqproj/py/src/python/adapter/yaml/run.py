import cattrs.preconf.pyyaml

from ...run import Run, Layout


def register_hooks(converter: cattrs.Converter) -> cattrs.Converter:
    def unstructure(run: Run) -> dict:
        dictionary = {
            "ind": run.ind,
            "layout": converter.unstructure(run.layout),
        }

        if run.machine:
            dictionary["machine"] = run.machine
        if run.reads:
            dictionary["reads"] = run.reads
        if run.bases:
            dictionary["bases"] = run.bases
        if run.description:
            dictionary["description"] = run.description
        return dictionary

    def structure(data: dict, ttype: type) -> Run:
        assert ttype is Run

        ind = converter.structure(data["ind"], str)
        layout = converter.structure(data["layout"], Layout)

        machine = converter.structure(data["machine"], str) if "machine" in data else None
        reads = converter.structure(data["reads"], int) if "reads" in data else None
        bases = converter.structure(data["bases"], int) if "bases" in data else None
        description = converter.structure(data["description"], str) if "description" in data else None
        return Run(ind, layout, machine, reads, bases, description)

    converter.register_unstructure_hook(Run, unstructure)
    converter.register_structure_hook(Run, structure)
    return converter
