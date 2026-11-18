use crate::scalar;

#[inline(always)]
pub(super) fn unpack_to_slice<ScalarLayout, ScalarStore>(words: &[u64], scalars: &mut [ScalarStore])
where
    ScalarLayout: scalar::Layout<Store = ScalarStore>,
    ScalarStore: scalar::Store,
{
    let width = ScalarLayout::BITS.get();
    let scalars_per_word = u64::BITS as usize / width;
    let full_words = scalars.len() / scalars_per_word;
    let full_scalars = full_words * scalars_per_word;
    let (scalars, tail) = scalars.split_at_mut(full_scalars);

    for (word, scalars) in words
        .iter()
        .take(full_words)
        .copied()
        .zip(scalars.chunks_exact_mut(scalars_per_word))
    {
        drain_word::<ScalarStore>(word, width, scalars);
    }

    if !tail.is_empty() {
        drain_word::<ScalarStore>(words[full_words], width, tail);
    }
}

macro_rules! impl_unpack_to_array {
    ($($fn:ident => $scalar_store:ty),+ $(,)?) => {
        $(
            #[allow(dead_code)]
            pub(super) const unsafe fn $fn<ScalarLayout, const LEN: usize>(
                words: &[u64],
            ) -> [$scalar_store; LEN]
            where
                ScalarLayout: scalar::Layout<Store = $scalar_store>,
            {
                let width = ScalarLayout::BITS.get();
                let mut scalars = [<$scalar_store as scalar::Store>::ZERO; LEN];
                if LEN == 0 {
                    return scalars;
                }

                let mut pos = 0;
                let mut word = 0;

                if width == u64::BITS as usize {
                    while pos < LEN {
                        scalars[pos] = words[word] as $scalar_store;
                        word += 1;
                        pos += 1;
                    }

                    return scalars;
                }

                let shift = u64::BITS as usize - width;
                let mut current = words[0];
                let mut remaining = u64::BITS as usize;

                while pos < LEN {
                    if remaining == 0 {
                        word += 1;
                        current = words[word];
                        remaining = u64::BITS as usize;
                    }

                    scalars[pos] = (current >> shift) as $scalar_store;
                    current <<= width;
                    remaining -= width;
                    pos += 1;
                }

                scalars
            }
        )+
    };
}

impl_unpack_to_array!(
    unpack_to_array_u8 => u8,
    unpack_to_array_u16 => u16,
    unpack_to_array_u32 => u32,
    unpack_to_array_u64 => u64,
);

#[inline(always)]
fn drain_word<ScalarStore>(word: u64, width: usize, scalars: &mut [ScalarStore])
where
    ScalarStore: scalar::Store,
{
    if width == u64::BITS as usize {
        for scalar in scalars {
            *scalar = unsafe { ScalarStore::from_u64_unchecked(word) };
        }

        return;
    }

    let shift = u64::BITS as usize - width;
    let mut word = word;

    for scalar in scalars {
        *scalar = unsafe { ScalarStore::from_u64_unchecked(word >> shift) };
        word <<= width;
    }
}
