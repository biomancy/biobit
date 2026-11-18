//! Bit-packed scalar storage primitives.
//!
//! `biobit-bitpack` stores fixed-width codec scalars densely inside small
//! integer stores, fixed arrays of `u64` words, or a growable `Vec<u64>`.
//! Users can use the built-in [`Dna`] codec or implement [`Codec`] to map
//! their own symbols to compact [`Scalar`] values.
//!
//! The core containers are:
//!
//! - [`PackedArray<C, L>`]: fixed-length, type-level length, with static store
//!   transitions for push and drop operations.
//! - [`PackedBuffer<C, S>`]: fixed-capacity runtime length over a fixed store
//!   type such as `u64` or `[u64; 2]`.
//! - [`PackedVec<C>`]: growable runtime length backed by `Vec<u64>`.
//!
//! # Bit alignment
//!
//! - A standalone [`Scalar`] is **low-aligned**. Its codeword occupies the
//!   least significant bits of its integer, with zeros above it.
//! - A packed sequence is **high-aligned** in its backing store. Element
//!   `0` occupies the most significant bits, and later elements follow
//!   toward the least significant bits. Unused low bits are zero.
//!
//! The alignment describes numeric bit positions independently of byte order.
//! For example, assuming a `u8` buffer and a two-bit DNA alphabet:
//! ```text
//! Bit pairs         7–6   5–4   3–2   1–0
//! Buffer before      00    01    10    00   high-aligned A, C, G, padding
//! Scalar T           00    00    00    11   low-aligned T
//! Buffer after       00    01    10    11   A, C, G, T
//! ```
//!
//! Most users should work through the safe container APIs. The `*_unchecked`
//! methods and low-level [`store`] traits exist so frontends can delegate to
//! optimized storage operations without repeating checks in hot paths.
//!
//! ```
//! use biobit_bitpack::{Codec, Dna, PackedBuffer};
//!
//! let scalars = [Dna::A, Dna::C, Dna::G].map(|base| Dna::encode(&base));
//! let mut buffer = PackedBuffer::<Dna, u8>::try_from_slice(&scalars).unwrap();
//! assert_eq!(*buffer.store(), 0b00_01_10_00);
//!
//! let t = Dna::encode(&Dna::T);
//! assert_eq!(t.store(), 0b00_00_00_11);
//!
//! assert!(buffer.push_right(t));
//! assert_eq!(buffer.len(), 4);
//! assert_eq!(*buffer.store(), 0b00_01_10_11);
//! ```

pub mod array;
pub mod buffer;
pub mod codec;
pub mod len;
pub mod scalar;
pub mod store;
pub mod vector;

pub use array::PackedArray;
pub use buffer::PackedBuffer;
pub use codec::{BitComplement, Bits, Codec, Dna};
pub use len::{Len, Length, Next, Prev};
pub use scalar::Scalar;
pub use vector::PackedVec;
