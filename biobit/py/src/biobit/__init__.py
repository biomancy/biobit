import sys as _sys
from importlib import import_module as _import_module
from importlib.machinery import ModuleSpec as _ModuleSpec
from types import ModuleType as _ModuleType

from . import collections, core, toolkit


def _masquerade_native_imports(module: _ModuleType, public_name: str) -> None:
    """Make biobit.rs.* modules appear as biobit.* by aliasing the import registry.

    The extension registers modules under names such as biobit.rs.toolkit.bioproj.
    Assigning those modules to public attributes allows attribute access, but a
    dotted import such as `from biobit.toolkit.bioproj.io.json import read` also
    needs the public module names in sys.modules. Native submodules have no Python
    implementation files for the import machinery to discover on disk.

    We deliberately alias each exported submodule in sys.modules, attach it to
    its public parent, and give it public import metadata for introspection. The
    original biobit.rs.* entries remain registered: both import paths share the
    same objects, and existing references and native class pickle paths still work.
    """
    _sys.modules[public_name] = module
    parent_name, _, child_name = public_name.rpartition(".")
    setattr(_sys.modules[parent_name], child_name, module)
    is_package = hasattr(module, "__path__")
    module.__name__ = public_name
    module.__package__ = public_name if is_package else parent_name
    module.__spec__ = _ModuleSpec(public_name, loader=None, is_package=is_package)
    for export in module.__all__:
        child = getattr(module, export)
        if isinstance(child, _ModuleType):
            _masquerade_native_imports(child, f"{public_name}.{export}")


# The I/O initializer retains its Python protocols and registers native streams.
# Import it after defining the helper it uses to expose the native package tree.
io = _import_module(f"{__name__}.io")

# These toolkit packages are entirely native; their Python files provide stubs.
# Other toolkit packages retain their normal Python implementations and paths.
for _package in ("bioproj", "reaper"):
    _masquerade_native_imports(
        _import_module(f"{__name__}.rs.toolkit.{_package}"),
        f"{__name__}.toolkit.{_package}",
    )

__all__ = ["collections", "core", "io", "toolkit"]
