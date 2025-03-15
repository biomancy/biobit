## Rust

### Module Organization

- Avoid creating a `prelude.rs`. If necessary, limit its use to anonymous trait re-exports (`as _`) to conveniently
  access trait methods through a single import, while still requiring explicit imports to use the traits directly.
- `mod.rs` should explicitly re-export all essential structs, functions, and traits.

This setup promotes clear module paths when naming structs and traits (e.g., `fasta::Record` rather than
`FastaRecord`). At the same time, `use crate::prelude::*` provides convenient access to trait methods without
polluting the namespace.

### `*Op` Traits

`*Op` traits abstract operations for structures sharing similar behaviors. For example, the `IntervalOp` trait defines
operations common to interval-like structures. This abstraction prevents code duplication and allows consistent
operation implementations across various `XOp` types.

### Why Require `Clone` for Certain Traits and Structures?

Rust’s strict ownership and memory-safety guarantees can complicate data management, especially across threads or Python
interfaces. To simplify this, certain generic structures and traits in `biobit` mandate implementing `Clone`. This
design places ownership decisions clearly in users' hands, allowing them to explicitly choose when to clone data or use
smart pointers to minimize overhead. Although this introduces minor runtime costs, it significantly simplifies
maintaining memory safety.

### Struct-Buffer-Builder Triangle

Many library structures, especially those dealing with IO operations, have three distinct states:

- **Structure:** Fully defined records meeting all invariants.
- **Buffer:** A thread-safe, reference-erased memory buffer (`Send + Sync`) for efficient caching.
- **Builder:** Fully typed but partially initialized structures.

Ideally, these variants should have identical memory layouts to enable zero-cost conversions between states. Currently,
this is not guaranteed, requiring manual implementation for Buffers and Builders. Future work will introduce derive
macros to automate and streamline this process.

## Python

### `Send` and `Sync` for Python Classes

With the introduction of free-threaded Python, the library assumes all Python objects can be safely shared and
transferred between threads without explicit Python-side synchronization. Consequently, Python objects are treated as
`Send` and `Sync`, an assumption generally enforced by PyO3.

### Why Not Implement `Clone` or `Copy` for All Python Classes?

This limitation is addressed in [this PyO3 issue](https://github.com/PyO3/pyo3/issues/4337). Once resolved, suitable
Python classes will implement `Clone` and/or `Copy` traits.

### Organizing Python Code per Submodule (e.g., `modules/*/py`)

Currently, dependencies among PyO3 modules are poorly supported (
see [this PyO3 issue](https://github.com/PyO3/pyo3/issues/1444)). Specifically, common dependencies may compile multiple
times, resulting in incompatible class versions. A temporary workaround involves creating and populating all `PyModule`
instances within a single umbrella `lib.rs` file.

Local Python dependencies also present challenges (
see [StackOverflow discussion](https://stackoverflow.com/questions/75159453/specifying-local-relative-dependency-in-pyproject-toml)).
Presently, the workaround is to symlink sources for each module within an overarching Python project.

## Additional Notes

- The `Interval` struct intentionally excludes a `data` field to simplify operations like `merge`, avoiding complexities
  related to arbitrary data updates.
- Read operations commonly accept mutable buffers to promote efficient reuse and minimize repeated allocations.
- A long-term goal is to stimulate batch-oriented processing, particularly in Python wrappers.

## Future Ideas

- Introduce a `stitch(max_gap: usize)` method for inverted repeats, enabling seamless merging of adjacent repeats
  separated by gaps smaller than `max_gap`.

### The `Bundle` Trait

Logical grouping of structures (e.g., by strand or seqid) is frequently required. Initially, a dedicated `Bundle<T>`
struct was considered but had several drawbacks:

- Primarily functioned as a thin wrapper around collections like `HashMap` or `BTreeMap`, adding minimal additional
  functionality.
- Lacked clear responsibilities beyond basic data storage.
- Was inconvenient for Python users, requiring interaction with specialized Rust structures instead of familiar Python
  types like `dict`.

Instead of creating a standalone struct, a `Bundle` trait has been implemented across multiple collection types (
`HashMap`, `BTreeMap`, `Vec`, etc.). This trait provides standardized methods (`get`, `remove`, `iter`) for uniform data
access and integrates smoothly with Python wrappers through the dedicated `IntoBundle` struct, which can be constructed
directly from Python types (`dict`, `list`, `tuple`).
