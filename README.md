Why can't we have python code per submodule (i.e. inside modules/*/py)?
Because dependencies between py modules aren't really supported at the
moment (https://github.com/PyO3/pyo3/issues/1444).
As an example, in such a layout the py-core would be compiled for every single py module, giving rise to N differnet
Segment py-classes that are incompatible with each other.

Besides, local python dependencies aren't really supported at the moment: https://stackoverflow.com/questions/75159453/specifying-local-relative-dependency-in-pyproject-toml

Why don't all py classes implement Clone/Copy? https://github.com/PyO3/pyo3/issues/4337