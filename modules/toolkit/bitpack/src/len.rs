use std::num::NonZeroUsize;

mod sealed {
    pub trait Supported {}
}

/// Type-level marker for a fixed length.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Len<const N: usize>;

/// Supported non-zero length.
pub trait Length: sealed::Supported + Copy + Send + Sync + 'static {
    type Array<T>: AsMut<[T]> + AsRef<[T]>;

    const LEN: NonZeroUsize;

    fn array_from_fn<T, F>(cb: F) -> Self::Array<T>
    where
        F: FnMut(usize) -> T;
}

impl<const N: usize> Length for Len<N>
where
    Len<N>: sealed::Supported,
{
    type Array<T> = [T; N];

    const LEN: NonZeroUsize = NonZeroUsize::new(N).unwrap();

    #[inline(always)]
    fn array_from_fn<T, F>(cb: F) -> Self::Array<T>
    where
        F: FnMut(usize) -> T,
    {
        std::array::from_fn(cb)
    }
}

/// Supported successor length.
pub trait Next: Length {
    type Next: Length;
}

/// Supported predecessor length.
pub trait Prev: Length {
    type Prev: Length;
}

include!(concat!(env!("OUT_DIR"), "/bitpack_len_impls.rs"));
