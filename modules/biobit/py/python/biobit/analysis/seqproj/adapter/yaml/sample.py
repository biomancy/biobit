import cattrs.preconf.pyyaml

from ...sample import Sample


def register_hooks(converter: cattrs.Converter) -> cattrs.Converter:
    def unstructure(sample: Sample) -> dict:
        dictionary = {
            "ind": sample.ind,
            "organism": tuple(sample.organism),
        }
        if sample.attributes:
            dictionary["attributes"] = converter.unstructure(sample.attributes)
        if sample.description:
            dictionary["description"] = sample.description
        return dictionary

    def structure(data: dict, ttype: type) -> Sample:
        assert ttype is Sample

        ind = converter.structure(data["ind"], str)
        organism = converter.structure(data["organism"], set[str])
        attributes = converter.structure(data["attributes"], dict[str, str]) if "attributes" in data else {}
        description = converter.structure(data["description"], str) if "description" in data else None

        return Sample(ind, organism, attributes, description)

    converter.register_unstructure_hook(Sample, unstructure)
    converter.register_structure_hook(Sample, structure)
    return converter
