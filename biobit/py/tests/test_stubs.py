"""Supplement mypy.stubtest with checks for runtime type aliases and exports.

Our pinned mypy 2.4 cannot unwrap PEP 695 TypeAliasType objects to compare their
targets with the stubs. Those aliases are therefore excluded in stubtest.allowlist.
This test supplies the missing checks: for each `type Alias = ...` declaration in
a public .pyi file, it compares the runtime target with the independently declared
stub expression. The runtime may use a PEP 695 alias or an ordinary assignment;
only PEP 695 aliases need their targets unwrapped through __value__. For example,
`type Dataset = Fastq | PairedFastq` must include both classes at runtime, including
when Rust constructs the alias during module registration.

Each stub is parsed as an AST without executing its class or function definitions.
Alias expressions are evaluated in a copy of the runtime module's namespace,
supplemented with ordinary imports from the stub, such as os for os.PathLike.
The test also checks that __all__ matches exactly, including order, and that every
listed export exists. These export checks overlap with stubtest's checks.
"""

import ast
from importlib import import_module
from pathlib import Path
from typing import TypeAliasType

import pytest

import biobit

PACKAGE = Path(biobit.__file__).resolve().parent
STUBS = sorted(PACKAGE.rglob("*.pyi", recurse_symlinks=True))


@pytest.mark.parametrize("path", STUBS, ids=lambda path: str(path.relative_to(PACKAGE)))
def test_exports_and_type_aliases_match_the_stub(path: Path) -> None:
    parts = path.relative_to(PACKAGE).with_suffix("").parts
    if parts[-1] == "__init__":
        parts = parts[:-1]
    module = import_module(".".join((biobit.__name__, *parts)))
    declarations = ast.parse(path.read_text(), filename=str(path))

    exports = next(
        statement.value
        for statement in declarations.body
        if isinstance(statement, ast.Assign)
        and any(
            isinstance(target, ast.Name) and target.id == "__all__"
            for target in statement.targets
        )
    )
    assert module.__all__ == ast.literal_eval(exports)
    assert all(hasattr(module, name) for name in module.__all__)

    # A stub can import modules for its alias expressions without requiring
    # those imports in the runtime implementation (for example, os.PathLike).
    namespace = vars(module).copy()
    imports = ast.Module(
        body=[
            statement
            for statement in declarations.body
            if isinstance(statement, ast.Import)
        ],
        type_ignores=[],
    )
    # Execute only import statements from our own checked-in stubs.
    exec(compile(imports, str(path), "exec"), namespace)  # noqa: S102

    # mypy 2.4 stubtest does not unwrap PEP 695 aliases. Compare their actual
    # targets against the independently maintained stub expressions here.
    for statement in declarations.body:
        if isinstance(statement, ast.TypeAlias):
            alias = getattr(module, statement.name.id)
            if isinstance(alias, TypeAliasType):
                alias = alias.__value__
            expression = ast.Expression(statement.value)
            expected = eval(compile(expression, str(path), "eval"), namespace)
            assert alias == expected, statement.name.id
