#[inline(always)]
pub(crate) const fn low_mask(bits: usize) -> u64 {
    match bits {
        0 => 0,
        64 => u64::MAX,
        _ => (1u64 << bits) - 1,
    }
}

#[inline(always)]
pub(crate) const fn high_mask(bits: usize) -> u64 {
    match bits {
        0 => 0,
        64 => u64::MAX,
        _ => u64::MAX << (u64::BITS as usize - bits),
    }
}

#[inline(always)]
pub(crate) const fn copy_prefix<const FROM: usize, const TO: usize>(
    from: &[u64; FROM],
) -> [u64; TO] {
    let mut to = [0; TO];
    let mut word = 0;

    while word < FROM && word < TO {
        to[word] = from[word];
        word += 1;
    }

    to
}

#[inline(always)]
pub(crate) const fn u64_to_words(store: u64) -> [u64; 2] {
    [store, 0]
}

#[inline(always)]
pub(crate) const fn get_unchecked(words: &[u64], width: usize, pos: usize) -> u64 {
    let bit = pos * width;
    let word = bit / u64::BITS as usize;
    let offset = bit % u64::BITS as usize;
    let available = u64::BITS as usize - offset;

    if width <= available {
        words[word] >> (available - width)
    } else {
        let lower = width - available;

        (words[word] << lower) | (words[word + 1] >> (u64::BITS as usize - lower))
    }
}

#[inline(always)]
pub(crate) const fn set_unchecked(words: &mut [u64], width: usize, pos: usize, scalar: u64) {
    let bit = pos * width;
    let word = bit / u64::BITS as usize;
    let offset = bit % u64::BITS as usize;
    let available = u64::BITS as usize - offset;

    if width <= available {
        let shift = available - width;
        let mask = low_mask(width) << shift;
        let scalar = scalar << shift;

        words[word] = (words[word] & !mask) | scalar;
    } else {
        let lower = width - available;

        let high_mask = low_mask(available);
        let high = scalar >> lower;
        words[word] = (words[word] & !high_mask) | high;

        let low_mask = low_mask(lower) << (u64::BITS as usize - lower);
        let low = scalar << (u64::BITS as usize - lower);
        words[word + 1] = (words[word + 1] & !low_mask) | low;
    }
}

#[inline(always)]
pub(crate) const fn clear_after(words: &mut [u64], used_bits: usize) {
    let full_words = used_bits / u64::BITS as usize;
    let tail_bits = used_bits % u64::BITS as usize;
    let mut word = full_words;

    if tail_bits != 0 {
        words[word] &= high_mask(tail_bits);
        word += 1;
    }

    while word < words.len() {
        words[word] = 0;
        word += 1;
    }
}

#[inline(always)]
pub(crate) const fn shift_left(words: &mut [u64], bits: usize) {
    if bits == 0 {
        return;
    }

    let word_shift = bits / u64::BITS as usize;
    let bit_shift = bits % u64::BITS as usize;
    let mut dst = 0;

    while dst < words.len() {
        let src = dst + word_shift;
        let mut value = 0;

        if src < words.len() {
            value = words[src] << bit_shift;

            if bit_shift != 0 && src + 1 < words.len() {
                value |= words[src + 1] >> (u64::BITS as usize - bit_shift);
            }
        }

        words[dst] = value;
        dst += 1;
    }
}

#[inline(always)]
pub(crate) const fn shift_right(words: &mut [u64], bits: usize) {
    if bits == 0 {
        return;
    }

    let word_shift = bits / u64::BITS as usize;
    let bit_shift = bits % u64::BITS as usize;
    let mut dst = words.len();

    while dst > 0 {
        dst -= 1;

        let mut value = 0;

        if dst >= word_shift {
            let src = dst - word_shift;

            value = if bit_shift == 0 {
                words[src]
            } else {
                words[src] >> bit_shift
            };

            if bit_shift != 0 && src > 0 {
                value |= words[src - 1] << (u64::BITS as usize - bit_shift);
            }
        }

        words[dst] = value;
    }
}
