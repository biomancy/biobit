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

/// Position in a packed sequence of 64-bit words.
pub struct WordOffset {
    /// u64 word index within the store.
    pub index: usize,
    /// Number of bits from the most significant end of the word.
    pub offset: usize,
}

impl WordOffset {
    /// Locates an element with a width of 1 through 64 bits.
    #[inline(always)]
    pub const fn new(element_index: usize, bits_per_element: usize) -> Self {
        // Each group of 64 elements occupies exactly bits_per_element whole words.
        let groups = element_index / 64;
        let tail_bits = (element_index % 64) * bits_per_element;
        Self {
            index: groups * bits_per_element + tail_bits / 64,
            offset: tail_bits % 64,
        }
    }

    #[inline(always)]
    pub const fn available(&self) -> usize {
        u64::BITS as usize - self.offset
    }
}

#[inline(always)]
pub(crate) const fn get_unchecked(words: &[u64], width: usize, pos: usize) -> u64 {
    let start = WordOffset::new(pos, width);
    let available = start.available();

    if width <= available {
        words[start.index] >> (available - width)
    } else {
        let lower = width - available;

        (words[start.index] << lower) | (words[start.index + 1] >> (u64::BITS as usize - lower))
    }
}

#[inline(always)]
pub(crate) const fn set_unchecked(words: &mut [u64], width: usize, pos: usize, scalar: u64) {
    let start = WordOffset::new(pos, width);
    let available = start.available();

    if width <= available {
        let shift = available - width;
        let mask = low_mask(width) << shift;
        let scalar = scalar << shift;

        words[start.index] = (words[start.index] & !mask) | scalar;
    } else {
        let lower = width - available;

        let high_mask = low_mask(available);
        let high = scalar >> lower;
        words[start.index] = (words[start.index] & !high_mask) | high;

        let low_mask = low_mask(lower) << (u64::BITS as usize - lower);
        let low = scalar << (u64::BITS as usize - lower);
        words[start.index + 1] = (words[start.index + 1] & !low_mask) | low;
    }
}

#[inline(always)]
pub(crate) const fn clear_after(words: &mut [u64], len: usize, width: usize) {
    let end = WordOffset::new(len, width);
    let mut word = end.index;

    if end.offset != 0 {
        words[word] &= high_mask(end.offset);
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

#[cfg(test)]
mod tests {
    use super::WordOffset;

    #[test]
    fn word_offsets_match_known_positions() {
        // Columns are element widths. Each cell is (word index, bit offset).
        let widths = [1, 3, 32, 63, 64];
        let max = usize::MAX;
        let q = max / 64; // max = 64q + 63 on every supported target.
        let rows = [
            (0, [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)]),
            (1, [(0, 1), (0, 3), (0, 32), (0, 63), (1, 0)]),
            (21, [(0, 21), (0, 63), (10, 32), (20, 43), (21, 0)]),
            (63, [(0, 63), (2, 61), (31, 32), (62, 1), (63, 0)]),
            (64, [(1, 0), (3, 0), (32, 0), (63, 0), (64, 0)]),
            (65, [(1, 1), (3, 3), (32, 32), (63, 63), (65, 0)]),
            (
                max,
                [
                    (q, 63),
                    (3 * q + 2, 61),
                    (max / 2, 32),
                    (max - q - 1, 1),
                    (max, 0),
                ],
            ),
        ];

        for (index, row) in rows {
            for (width, expected) in widths.into_iter().zip(row) {
                let actual = WordOffset::new(index, width);
                assert_eq!(
                    (actual.index, actual.offset),
                    expected,
                    "index={index}, width={width}"
                );
            }
        }
    }
}
