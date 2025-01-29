from typing import cast

import cattrs.preconf.pyyaml

from ...layout import Layout, MatesOrientation


def register_hooks(converter: cattrs.Converter) -> cattrs.Converter:
    def unstructure(run: Layout) -> dict:
        if isinstance(run, Layout.Single):
            return {
                "type": "single",
                "file": run.file
            }
        elif isinstance(run, Layout.Paired):
            return {
                "type": "paired",
                "files": converter.unstructure(run.files),
                "orientation": str(run.orientation) if run.orientation else None,
            }
        else:
            raise TypeError(f"Unsupported layout type: {run}")

    def structure(data: dict, ttype: type) -> Layout:
        assert ttype is Layout

        if data["type"] == "single":
            return Layout.Single(data["file"])
        elif data["type"] == "paired":
            files: tuple[str, str] = cast(tuple[str, str], converter.structure(data["files"], tuple[str, str]))
            orientation = MatesOrientation(data["orientation"]) if data["orientation"] else None
            return Layout.Paired(orientation, files)
        else:
            raise ValueError(f"Unsupported layout type: {data}")

    converter.register_unstructure_hook(Layout, unstructure)
    converter.register_structure_hook(Layout, structure)
    return converter
