## Rust

### How are modules organized?

- The `prelude.rs` usage is strongly discouraged. If necessary, it should be used only for anonymous re-exports (`as _`)
  of traits providing access to all trait methods through a single import. This requires the use of fully qualified
  names when referencing the trait itself.
- `mod.rs` should re-export all public structures, functions, and traits.

This structure allows module paths to be part of trait or struct names (e.g., `my_struct::Builder` instead of
`MyStructBuilder`). At the same time, using `use my_struct::prelude::*` provides access to all trait methods without
polluting the current namespace.

### What is the `Bundle` trait?

Grouping structures logically (e.g., by strand or chromosome) is often necessary. Initially, a `Bundle<T>` struct was
considered, where `T` represents the bundled structure type. However, this approach has several drawbacks:

- It serves primarily as a wrapper around `HashMap` or `BTreeMap`, adding little significant functionality.
- The `Bundle` struct does not have a clear purpose beyond data storage.
- It offers limited utility in Python wrappers, as it compels users to work with a `Bundle` struct instead of more
  familiar Python `dict` structures.

Consequently, the decision is made to avoid introducing a `Bundle` struct and instead implement a `Bundle` trait for
several bundle-like structures, such as `HashMap`, `BTreeMap`, and `Vec`. This trait includes only essential methods,
like `get`, `remove`, and `iter`, for data interaction. The design integrates well with Python wrappers, where the
`Bundle` trait is implemented for an `IntoBundle` struct, which can be constructed from Python types like `dict`,
`list`, or `tuple`.

The `Bundle` trait is preferred over a `Bundle` struct to maintain flexibility and ease of future modifications. It may
become evident that a `Bundle` struct is needed, but only time will tell.

### What are `*Op` traits?

`*Op` traits are defined for operations applicable to structures that behave like `X`. For instance, `IntervalOp`
specifies operations that can be performed on an `Interval`-like structure. This approach allows the implementation of
these operations across multiple structures, such as `Interval` and `MySuperInterval`, without code duplication.

### Why is `Clone` required for generic parameters in certain traits and structures?

In Rust, ownership and memory safety are strictly enforced, but managing ownership across multiple threads or when
interfacing with external languages (like Python) can become complex. To simplify this, some generic structures and
traits in the `biobit` library require their parameters to implement the `Clone` trait.

By enforcing `Clone`, these structures offload ownership management to the user, who can either create copies of the
data when needed or use smart pointers to avoid deep cloning. While this may introduce a slight performance overhead, it
serves as a practical default for ensuring safe memory handling.

For more efficiency, users are encouraged to adopt a `Stash`/`Unstash` strategy. With this approach, the structure is
temporarily stored in a type- and lifetime-erased form within a `Stash` and can be reconstructed on demand. In other
words, you stash the structure when it's not actively used, create a reference-tracked version when needed, and stash it
again when done. This minimizes unnecessary cloning while keeping memory management flexible and efficient.

(TODO: Implement `Stash`/`Unstash` in the library)

## Python

### Why are all Python classes `Send` and `Sync`?

Free-threaded Python is expected to become production-ready with Python 3.14/3.15. To future-proof the library, it is
assumed that all Python objects can be accessed and sent between threads without requiring Python-side synchronization.

An alternative approach would assume synchronized access to Rust wrappers on the Python side. However, users may
overlook or forget this requirement, leading to subtle, hard-to-diagnose bugs. Users seeking maximum performance can
utilize the Rust interface directly, while others must accept the overhead of Python wrapper synchronization.

### Why don't all Python classes implement `Clone`/`Copy`?

The limitation is detailed in [this PyO3 issue](https://github.com/PyO3/pyo3/issues/4337). Once this issue is resolved,
`Clone`/`Copy` will be implemented for all classes that can benefit from these traits.

### Why can’t Python code be organized per submodule (e.g., inside `modules/*/py`)?

Currently, dependencies between Python modules are not well-supported (
see [this PyO3 issue](https://github.com/PyO3/pyo3/issues/1444)). In this structure, core Python components (like
`py-core`) would be compiled separately for each module, leading to multiple incompatible versions of certain classes,
such as `Interval`.

Additionally, local Python dependencies are poorly supported at present. More details can be found
in [this StackOverflow discussion](https://stackoverflow.com/questions/75159453/specifying-local-relative-dependency-in-pyproject-toml).

If these issues are resolved in the future, the possibility of relocating Python code to `modules/*/py` directories and
re-exporting it in the core Python module, possibly behind feature flags, can be revisited.

### Everything is a reference

In Python, (almost) everything is a reference. This principle is reflected in most Python classes, often necessitating
that internal Rust structures are encapsulated behind an `Arc` and either a `Mutex` or `RwLock`. (Remember, every class
is also `Sync` and `Send`.)

The performance penalty of this approach is negligible in most cases, as the structures need to be `Sync`/`Send` anyway.
This design avoids introducing confusing behavior that would be unexpected for Python users.

Exceptions to this rule are clearly marked in the codebase and documentation.

# Random notes:

- Including a `data` field in the `Interval` struct complicates operations like `merge`, as the `data` field must be
  updated. It's the reason why there is no `data` field in the `Segment` (and other) structures.
- All traits and structs should prioritize batch-oriented processing, especially for Python wrappers, where batch
  operations are significantly faster than repeated method invocations. Where possible, batch methods should accept data
  via slices and the result buffer by a mutable reference.

# Ideas
- Add a `stitch(max_gap: usize)` method to inverted repeats to merge adjacent repeats with a gap smaller than `max_gap`.