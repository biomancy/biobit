---
A draft design document.
---

The library has adopted **Apache Arrow** as its primary in-memory format for batch data processing. This decision
enables zero-copy data sharing between the high-performance Rust core and language bindings like Python. Most data
structures are implemented as **columnar batches**, where each record field is represented as a single column (a
structure-of-arrays, or SoA).

---

## API Design: The Two-Type System

To ensure safety and clarity, the API is built on a two-type system that separates immutable and mutable states.
Functionality is scoped within modules (e.g., `module::Batch`).

* **`module::Batch`**: The default, **immutable**, and shareable type. It's designed for all read-only operations and
  queries. It is cheap to clone, as it only increments reference counts to the underlying data.
* **`module::BatchMut`**: A unique, owning, and **mutable** type. This type provides methods for efficient, in-place
  data modification with guaranteed exclusive access.

The conversion between types is framed as an explicit "editing session," ensuring user intent is clear across both the
Rust and Python APIs.

* **`.edit()`**: Converts an immutable `Batch` into a mutable `BatchMut`. This is a **safe** operation that uses a *
  *Copy-on-Write (CoW)** mechanism at the column level. A copy is only performed for columns that are actually mutated,
  and only when they are first modified.
* **`.commit()`**: Finalizes the editing session by consuming the `BatchMut` and returning a new, immutable `Batch`.
  This is a **zero-cost** operation.

The design avoids interior mutability; `BatchMut` enforces exclusive ownership via Rust's standard ownership model.
Language bindings enforce this by wrapping `BatchMut` instances in a `Mutex` to serialize access and ensure thread
safety.

Importantly, it's rarely possible to leverage CoW mechanism in bindings, because they don't destroy the original `Batch`
when calling `.edit()` thus leading to a shared ownership between CoW buffers in the `BatchMut` and the original
`Batch`. Or, even more often, the bindings will cache references to FFI-exposed buffers addressed at least once,
which also leads to shared ownership. This means that any mutation will always trigger a copy, leading to suboptimal
performance. See the [[Unsafe In-Place Mutation]] section for an escape hatch.

---

## Unsafe In-Place Mutation

For performance-critical paths, an escape hatch is provided to bypass the CoW mechanism and mutate data in place.

* **Rust**: `unsafe fn edit_inplace(self) -> BatchMut`
* **Python**: `unsafe_edit_inplace(self) -> BatchMut`

Calling these functions is a promise that the user holds unique references to all `Batch` columns. If this contract is
violated, other references will observe the mutated data, leading to a data race.

---

## Extension Trait Architecture

Functionality is delivered via a layered extension trait architecture. This allows `batch.method()` to work seamlessly,
mimicking the ergonomic APIs of libraries like PyTorch or Pandas.

* **`core::BatchExt`**: A generic trait for universal, read-only operations (e.g., `len()`, `slice()`).
* **`core::BatchMutExt`**: A generic trait for universal, in-place mutating operations (e.g., `shrink_to_fit()`).
* **`module::BatchExt`**: A specific trait for module-related, read-only operations. It is implemented for both
  `module::Batch` and `module::BatchMut`.
* **`module::BatchMutExt`**: A specific trait for module-related, in-place mutating operations, implemented only for
  `module::BatchMut`.
