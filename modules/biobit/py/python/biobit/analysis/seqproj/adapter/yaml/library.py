import cattrs.preconf.pyyaml

from ...library import Library, Stranding


def register_hooks(converter: cattrs.Converter) -> cattrs.Converter:
    def unstructure(lib: Library) -> dict:
        dictionary = {
            "source": tuple(lib.source),
            "selection": tuple(lib.selection),
            "stranding": converter.unstructure(lib.stranding),
        }

        if lib.attributes:
            dictionary["attributes"] = converter.unstructure(lib.attributes)
        return dictionary

    def structure(data: dict, ttype: type) -> Library:
        assert ttype is Library

        source = converter.structure(data["source"], set[str])
        selection = converter.structure(data["selection"], set[str])
        stranding = converter.structure(data["stranding"], Stranding)
        attributes = converter.structure(data["attributes"], dict[str, str]) if "attributes" in data else {}

        return Library(source, selection, stranding, attributes)

    converter.register_unstructure_hook(Library, unstructure)
    converter.register_structure_hook(Library, structure)
    return converter
