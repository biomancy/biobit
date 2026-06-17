# Repository Guidelines

## Project Structure & Module Organization

This repository is a Rust workspace for bioinformatics crates with Python bindings built through PyO3/maturin. Core Rust crates live under `modules/*/rs`, with matching Python binding crates under `modules/*/py`. The umbrella Python package is in `biobit/py`; docs sources are in `biobit/docs/src`. Test data and indexed genomics fixtures are kept in `resources/`. `obsolete/` contains historical code and should not be used as a template.

## Build, Test, and Development Commands

Use `pixi` from the repository root; task definitions are in `pixi.toml`.

- `pixi run ci`: run docs, Rust, and Python CI tasks.
- `pixi run -e rust rs-ci`: run `cargo fmt --check`, `cargo check`, `cargo clippy`, and Rust tests.
- `pixi run -e py-3-14t py-ci`: run Ruff, mypy, build the Python extension with `maturin develop`, then run pytest.
- `pixi run -e docs build-docs`: build the MkDocs site into `biobit/docs/site`.

## Coding Style & Naming Conventions

Rust uses edition 2024 and standard `cargo fmt` formatting. Keep Rust modules snake_case, public types UpperCamelCase, and tests close to their crate in `tests/` or module-level test modules. Python code targets Python 3.13+ and is checked with Ruff and mypy; use snake_case for functions/modules and UpperCamelCase for classes. Prefer typed Python APIs and keep PyO3 boundary code in `src/bindings` separate from pure Python package code in `src/python` or `biobit/py/src/biobit`.

## Testing Guidelines

Rust integration tests are under crate-local `tests/` directories, for example `modules/alignment/rs/tests`. Python tests follow `test_*.py` naming and are colocated under each Python module, such as `modules/core/py/src/python/.../tests`. Keep tests minimal: cover the affected public API and lock in observed behavior. Ask before adding resource-heavy integration tests that use BAM files, FASTA genomes, or substantial compute.

## Commit & Pull Request Guidelines

Recent commits use short descriptive subjects, often with PR numbers, for example `Regression test (#16)` or `seqproj refactoring (1) (#18)`. Keep titles focused on the affected module or behavior. Pull requests should describe the change, list the validation run (`pixi run ci` or narrower tasks), link relevant issues, and mention fixture or generated-file updates. Include screenshots only for documentation or rendered-site changes.

## Agent-Specific Notes

Do not overwrite generated caches or build artifacts such as `.ruff_cache`, `.pytest_cache`, `.mypy_cache`, `target/`, or `biobit/docs/site`. Preserve existing uncommitted user changes and keep edits scoped to the requested module.
