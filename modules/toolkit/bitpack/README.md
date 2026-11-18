# biobit-bitpack

`biobit-bitpack` is the low-level packed scalar crate for the biobit
workspace. It stores fixed-width codec scalars densely in primitive integers,
fixed arrays of `u64` words, or a growable `Vec<u64>`.

The crate is intended to remain independent from domain crates. Domain code
defines a `Codec` for symbols such as DNA bases, while this crate owns the
storage layouts, packing operations, and container APIs.

## Current Status

Implemented:

- `Codec`, `Bits<N>`, and `Scalar<C>` for codec-valid fixed-width values.
- Type-level lengths through `Len<N>`, `Length`, `Next`, and `Prev`.
- `PackedArray<C, L>` for fixed-length arrays with type-level length.
- `PackedBuffer<C, S>` for fixed-capacity, runtime-length buffers.
- `PackedVec<C>` for growable runtime-length packed vectors.
- Primitive stores: `u8`, `u16`, `u32`, `u64`.
- Word stores: `[u64; N]` for generated `N >= 2` layouts.
- Store operations for get, set, pack, unpack, reverse, complement, slide,
  push, drop, grow, and shrink.
- Generated scalar and array layout impls in `build.rs`.
- Generated fixed word-store impls in `build.rs`.

Not currently implemented:

- A slice-backed or borrowed packed view API.
- A fully generic packed string/k-mer layer.
- Serialization format guarantees. The current bit order is a value-level
  convention, not a byte format.
- SIMD specializations. The word-store code is structured so SIMD dispatch can
  be added later.

## Data Model

A user defines a `Codec`:

- `Codec::Symbol` is the semantic symbol type.
- `Codec::Bits` selects the number of bits per scalar.
- `Codec::encode` maps a symbol into a codec-valid `Scalar<C>`.
- `Codec::decode` maps a scalar back into a symbol.

`Scalar<C>` is a transparent wrapper around the generated scalar store
(`u8`, `u16`, `u32`, or `u64`). The scalar value is always low-bit aligned.

Packed containers store scalars in most-significant-bit order:

- position `0` occupies the most significant active scalar field,
- later positions move toward less significant bits,
- unused low-order padding bits are zero.

## Containers

### `PackedArray<C, L>`

Use this when length is known at compile time. The length is represented by
`Len<N>`. Push and drop operations return a different `PackedArray` type whose
length is the successor or predecessor length.

Example:

```rust
use biobit_bitpack::{Bits, Codec, Len, PackedArray, Scalar};

#[derive(Clone, Copy)]
struct TwoBit;

impl Codec for TwoBit {
    type Symbol = u8;
    type Bits = Bits<2>;

    fn encode(symbol: &Self::Symbol) -> Scalar<Self> {
        Scalar::new(*symbol).unwrap()
    }

    fn decode(scalar: Scalar<Self>) -> Self::Symbol {
        scalar.store()
    }
}

let scalars = [
    TwoBit::encode(&0),
    TwoBit::encode(&1),
    TwoBit::encode(&2),
];

let array = PackedArray::<TwoBit, Len<3>>::from_array(&scalars);
let array = array.push_right(TwoBit::encode(&3));

assert_eq!(array.to_array().map(TwoBit::decode), [0, 1, 2, 3]);
```

### `PackedBuffer<C, S>`

Use this for a fixed-capacity container with runtime length. The store type `S`
is fixed after creation, for example `u64` or `[u64; 2]`. Push operations are
fallible and return `false` when the buffer is full. Drop and slide operations
are fallible when the buffer is empty.

### `PackedVec<C>`

Use this for a growable runtime-length container. It stores words in a
`Vec<u64>`. Push operations grow the vector as needed. Drop operations truncate
unused words after removing scalars.

## Store Architecture

The `store` module defines the low-level operation traits. These traits are
unsafe because implementations must preserve the packed layout invariant.

The implementation families are split by storage backend:

- `store::primitives`: implementations for `u8`, `u16`, `u32`, and `u64`.
- `store::words`: implementations for `[u64; N]`, `[u64]`, and `Vec<u64>`
  style word storage helpers.

Public containers are thin frontends. They perform bounds, length, and capacity
checks, then delegate to the corresponding store operation.

`build.rs` generates the supported layout graph:

- `Len<1>` through `Len<128>`.
- `Next` and `Prev` transitions for supported lengths.
- scalar layouts for `Bits<1>` through `Bits<64>`.
- array layouts choosing the least capable static store for each
  `(bits per scalar, length)` pair.
- static store impls for `[u64; 2]` through `[u64; 128]`.

## Tests

The test layout is intentionally layered:

- `tests/array.rs` covers the public `PackedArray` API with hand-written
  scenarios. Randomized testing is not used because the type-level length
  changes across push and drop operations.
- `tests/buffer.rs` covers the public `PackedBuffer` API and uses a mirror
  runner for deterministic pseudo-random operation chains.
- `tests/vector.rs` covers the public `PackedVec` API and uses the same mirror
  runner.
- `tests/ops/` contains shared test-only operations, views, and codecs.
- `tests/ops/codec.rs` defines `Raw<N>`, a test codec where every masked
  codeword is valid. This keeps bitpack tests independent from domain crates.
- store module unit tests cover low-level implementation details and specialized
  bit-manipulation paths.

Domain crates should not test bitpack container correctness. For example,
`biobit-core` tests DNA codec semantics, while this crate tests packed
storage behavior using `Raw<N>`.

## Development Notes

Run the standard workspace checks with:

```sh
pixi run fmt
pixi run check
pixi run clippy
pixi run test
```

For quieter full test output during development:

```sh
pixi run cargo test --all-features -q
```

When resuming this project, start with the integration tests in
`crates/bitpack/tests/`. They describe the intended public API behavior more
clearly than the low-level store implementations.
