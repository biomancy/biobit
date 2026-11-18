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
//! All containers use the same logical bit order: position `0` is stored in the
//! most significant active scalar slot, and following positions move toward
//! less significant bits. Unused low-order padding bits are kept zero. This is
//! a value-level layout convention only; it is not a byte serialization format.
//!
//! Most users should work through the safe container APIs. The `*_unchecked`
//! methods and low-level [`store`] traits exist so frontends can delegate to
//! optimized storage operations without repeating checks in hot paths.
//!
//! # Example
//!
//! ```
//! use biobit_bitpack::{Codec, Dna, Len, PackedArray, PackedVec};
//!
//! let scalars = [
//!     Dna::encode(&Dna::A),
//!     Dna::encode(&Dna::C),
//!     Dna::encode(&Dna::G),
//!     Dna::encode(&Dna::T),
//! ];
//!
//! let array = PackedArray::<Dna, Len<4>>::from_array(&scalars);
//! assert_eq!(array.get(2).map(Dna::decode), Some(Dna::G));
//! assert_eq!(array.to_array().map(Dna::decode), [Dna::A, Dna::C, Dna::G, Dna::T]);
//!
//! let mut vector = PackedVec::<Dna>::from_array::<Len<4>>(&scalars);
//! vector.push_right(Dna::encode(&Dna::A));
//! assert_eq!(vector.len(), 5);
//! assert_eq!(vector.get(4).map(Dna::decode), Some(Dna::A));
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
