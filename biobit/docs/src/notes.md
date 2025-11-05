# Architecture Decisions

## IO primitives

TODO: Explain encode-decode pattern.

Read operations commonly accept mutable buffers to promote efficient reuse and minimize repeated allocations.

# Rust

## Module Public API (`mod.rs`)

Each module's `mod.rs` file explicitly defines its public interface by re-exporting all intended public
items (structs, enums, traits, functions, sub-modules) using `pub use`. This promotes clear discoverability (e.g.,
accessing `fasta::Record` requires importing the `fasta` module only) and establishes a stable contract for the
module's users.

## Optional Prelude (`prelude.rs`)

If a prelude module is used, its scope is strictly limited to re-exporting common *traits* anonymously
(`pub use TraitName as _;`). This allows convenient access to trait extension methods via a single glob import
(`use crate::prelude::*;`) without polluting the user's namespace with concrete type names, function names, or
even trait names themselves, thereby encouraging explicit imports for types and enhancing code clarity. Adherence to
this strict “traits-only, anonymous export” rule is crucial for effectiveness.

## Traits And Structures

Core data types (e.g., `Interval`, `Record`) are defined as concrete structs. Functionality is defined by implementing
fine-grained, *behavior-oriented* traits (e.g., `Spanning`, `Coverage`) directly on these structs. These traits define
specific, composable capabilities.

Common combinations of required traits are grouped using umbrella traits primarily to simplify trait bounds in generic
Rust functions and types. These aliases **must** be named based on the collective capability they represent (e.g.,
`ProcessableRead`, `AlignableSequence`) to promote abstraction and flexibility, not based on structural similarity to a
specific type (i.e., avoid names like `RecordLike` or `IntervalLike`).

Python bindings are created by wrapping concrete Rust structs (e.g., `PyInterval` wrapping `Interval`) using
`#[pyclass]`. These Python wrappers provide methods that mirror the functionality of the core behavioral traits
implemented by the underlying Rust struct, offering a Pythonic interface to capabilities like `Spanning` or `Coverage`.
This design also ensures that Python structs can be used in generic Rust functions, enabling easier interoperability
between Rust and Python.

For simplicity, all Rust traits have Python protocol equivalents that are leveraged for typing in Python bindings.

## Ownership Management Across FFI Boundaries

Rust’s ownership rules prevent returning borrowed references across FFI boundaries. When a function must reference input
features (e.g., pairing read counts with the original records), those references need to remain valid in FFI for an
indefinite period—thus borrowing (`&T`) is not feasible once data crosses the FFI boundary.

We address this in two ways:

1. **Move Owned Inputs (`T`):** If the function takes ownership of `T`, it can directly return that owned `T`. This is
   the most efficient option, but not always feasible if an algorithm must produce multiple or iterative results.
2. **Create Owned Copies (`T`):** An algorithm can produce multiple or iterative outputs for the same inputs by cloning
   them internally. This requires `T: Clone`.

The second option is used in some parts of the library. It does involve a performance penalty, but this can be mitigated
by using smart pointers or partial copies in Rust. In return, it provides more flexible and FFI-friendly APIs.

# Python

## `Send` and `Sync` for Python Classes

With the introduction of free-threaded Python, the library assumes all Python objects can be safely shared and
transferred between threads without explicit Python-side synchronization. Consequently, Python objects are treated as
`Send` and `Sync`, an assumption generally enforced by PyO3.

## Why Not Implement `Clone` or `Copy` for All Python Classes?

This limitation is discussed in [this PyO3 issue](https://github.com/PyO3/pyo3/issues/4337). Once resolved, suitable
Python classes will implement `Clone` and/or `Copy` traits.

## Organizing Python Code per Submodule (e.g., `modules/*/py`)

Currently, dependencies among PyO3 modules are poorly supported (see
[this PyO3 issue](https://github.com/PyO3/pyo3/issues/1444)). Specifically, common dependencies may compile multiple
times, resulting in incompatible class versions. A temporary workaround involves creating and populating all `PyModule`
instances within a single umbrella `lib.rs` file.

Local Python dependencies also present challenges (see
[StackOverflow discussion](https://stackoverflow.com/questions/75159453/specifying-local-relative-dependency-in-pyproject-toml)).
Presently, the workaround is to symlink sources for each module within an overarching Python project.

# Future Ideas

- A long-term goal is to stimulate batch-oriented processing, particularly in Python wrappers.

## The `Bundle` Trait

Logical grouping of structures (e.g., by strand or seqid) is frequently required. Initially, a dedicated `Bundle<T>`
struct was considered but had several drawbacks:

- Primarily functioned as a thin wrapper around collections like `HashMap` or `BTreeMap`, adding minimal additional
  functionality.
- Lacked clear responsibilities beyond basic data storage.
- Was inconvenient for Python users, requiring interaction with specialized Rust structures instead of familiar Python
  types like `dict`.

Instead of creating a standalone struct, a `Bundle` trait has been implemented across multiple collection types
(`HashMap`, `BTreeMap`, `Vec`, etc.). This trait provides standardized methods (`get`, `remove`, `iter`) for uniform
data access and integrates smoothly with Python wrappers through the dedicated `IntoBundle` struct, which can be
constructed directly from Python types (`dict`, `list`, `tuple`).

## Struct-Buffer-Builder Triangle

In ideal circumstances, many library structures, especially those dealing with IO operations, should have three distinct
states:

- **Structure:** Fully defined, meets all invariants.
- **Buffer:** A thread-safe, reference-erased memory buffer (`Send + Sync`) for efficient caching.
- **Builder:** Fully typed but partially initialized structures.

These variants should be memory-compatible to enable zero-cost conversions between states via
[TransmuteFrom](https://github.com/rust-lang/rust/issues/99571). Currently, `transmutability` is far from stabilization,
and a proper implementation of this pattern is not possible.

# Rejected Ideas

#### Stitching of Inverted Repeats

**Idea:** Introduce a `stitch(max_gap: usize)` method for inverted repeats, enabling merging of adjacent repeats
separated by gaps smaller than `max_gap`.

**Reason:** Impossible to implement without violating the underlying invariant. Stitching gaps will produce left/right
sides of the inverted repeat that might not be of the same length.

#### Data field for fundamental primitives

**Idea:** Introduce a `data` field in fundamental structures (e.g., `Interval`) to store additional information. It
could be generic with a default unit `()` type.

**Reason:** This would complicate the design and usage of these structures. For example, what should happen when merging
two `Interval` objects with different `data` types? Should the `data` field be merged or discarded? In the face of such
ambiguities, it is better to avoid the `data` field altogether.
