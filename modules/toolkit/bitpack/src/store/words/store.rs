use crate::store::Store;

/// Store backed by contiguous `u64` words in canonical packed bit order.
pub(crate) trait WordStore: Store {
    fn as_words(&self) -> &[u64];
}

/// Mutable store backed by contiguous `u64` words in canonical packed bit order.
pub(crate) trait WordStoreMut: WordStore {
    fn as_words_mut(&mut self) -> &mut [u64];
}

impl Store for [u64] {}

impl WordStore for [u64] {
    #[inline(always)]
    fn as_words(&self) -> &[u64] {
        self
    }
}

impl WordStoreMut for [u64] {
    #[inline(always)]
    fn as_words_mut(&mut self) -> &mut [u64] {
        self
    }
}

impl Store for Vec<u64> {}

impl WordStore for Vec<u64> {
    #[inline(always)]
    fn as_words(&self) -> &[u64] {
        self.as_slice()
    }
}

impl WordStoreMut for Vec<u64> {
    #[inline(always)]
    fn as_words_mut(&mut self) -> &mut [u64] {
        self.as_mut_slice()
    }
}

impl<const N: usize> WordStore for [u64; N]
where
    [u64; N]: Store,
{
    #[inline(always)]
    fn as_words(&self) -> &[u64] {
        self.as_slice()
    }
}

impl<const N: usize> WordStoreMut for [u64; N]
where
    [u64; N]: Store,
{
    #[inline(always)]
    fn as_words_mut(&mut self) -> &mut [u64] {
        self.as_mut_slice()
    }
}

include!(concat!(env!("OUT_DIR"), "/bitpack_word_store_impls.rs"));
