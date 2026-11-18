use crate::scalar;

pub(super) struct Decoder<'a> {
    words: &'a [u64],
    word: usize,
    current: u64,
    remaining: usize,
    width: usize,
}

impl<'a> Decoder<'a> {
    #[inline(always)]
    pub(super) fn new(words: &'a [u64], width: usize) -> Self {
        Self {
            words,
            word: 0,
            current: words[0],
            remaining: u64::BITS as usize,
            width,
        }
    }

    #[inline(always)]
    pub(super) fn next(&mut self) -> u64 {
        if self.remaining == 0 {
            self.word += 1;
            self.current = self.words[self.word];
            self.remaining = u64::BITS as usize;
        }

        if self.width <= self.remaining {
            let scalar = self.current >> (u64::BITS as usize - self.width);

            self.current <<= self.width;
            self.remaining -= self.width;

            scalar
        } else {
            let high_bits = self.remaining;
            let low_bits = self.width - high_bits;
            let high = self.current >> (u64::BITS as usize - high_bits);

            self.word += 1;
            self.current = self.words[self.word];
            self.remaining = u64::BITS as usize - low_bits;

            let low = self.current >> self.remaining;
            self.current <<= low_bits;

            (high << low_bits) | low
        }
    }
}

#[inline(always)]
pub(super) fn unpack_to_slice<ScalarLayout, ScalarStore>(words: &[u64], scalars: &mut [ScalarStore])
where
    ScalarLayout: scalar::Layout<Store = ScalarStore>,
    ScalarStore: scalar::Store,
{
    if scalars.is_empty() {
        return;
    }

    let mut decoder = Decoder::new(words, ScalarLayout::BITS.get());

    for scalar in scalars {
        *scalar = unsafe { ScalarStore::from_u64_unchecked(decoder.next()) };
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
                let mut current = words[0];
                let mut remaining = u64::BITS as usize;

                while pos < LEN {
                    let scalar;

                    if remaining == 0 {
                        word += 1;
                        current = words[word];
                        remaining = u64::BITS as usize;
                    }

                    if width <= remaining {
                        scalar = current >> (u64::BITS as usize - width);
                        current <<= width;
                        remaining -= width;
                    } else {
                        let high_bits = remaining;
                        let low_bits = width - high_bits;
                        let high = current >> (u64::BITS as usize - high_bits);

                        word += 1;
                        current = words[word];
                        remaining = u64::BITS as usize - low_bits;

                        let low = current >> remaining;
                        current <<= low_bits;

                        scalar = (high << low_bits) | low;
                    }

                    scalars[pos] = scalar as $scalar_store;
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
