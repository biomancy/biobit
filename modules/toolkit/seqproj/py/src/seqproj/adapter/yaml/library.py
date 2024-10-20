from typing import Any

import cattrs.preconf.pyyaml

from ...library import Library, Strandedness


def register_hooks(converter: cattrs.Converter) -> cattrs.Converter:
    def unstructure(lib: Library) -> dict:
        dictionary: dict[str, Any] = {
            "source": tuple(lib.source),
            "selection": tuple(lib.selection),
        }

        if lib.strandedness:
            dictionary["strandedness"] = str(lib.strandedness)
        if lib.attributes:
            dictionary["attributes"] = converter.unstructure(lib.attributes)
        return dictionary

    def structure(data: dict, ttype: type) -> Library:
        assert ttype is Library

        source = converter.structure(data["source"], set[str])
        selection = converter.structure(data["selection"], set[str])
        strandedness = Strandedness(data["strandedness"]) if "strandedness" in data else None
        attributes = converter.structure(data["attributes"], dict[str, str]) if "attributes" in data else {}

        return Library(source, selection, strandedness, attributes)

    converter.register_unstructure_hook(Library, unstructure)
    converter.register_structure_hook(Library, structure)
    return converter
