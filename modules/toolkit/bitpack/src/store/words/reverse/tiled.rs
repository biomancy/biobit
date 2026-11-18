#[inline(always)]
pub(super) const fn reverse_word(word: u64, width: usize) -> u64 {
    match width {
        1 => word.reverse_bits(),
        2 => reverse_width_2(word),
        4 => reverse_width_4(word),
        8 => word.swap_bytes(),
        16 => reverse_width_16(word),
        32 => word.rotate_left(32),
        64 => word,
        _ => word,
    }
}

#[inline(always)]
const fn reverse_width_2(word: u64) -> u64 {
    let word = ((word & 0xcccc_cccc_cccc_cccc) >> 2) | ((word & 0x3333_3333_3333_3333) << 2);
    let word = ((word & 0xf0f0_f0f0_f0f0_f0f0) >> 4) | ((word & 0x0f0f_0f0f_0f0f_0f0f) << 4);

    word.swap_bytes()
}

#[inline(always)]
const fn reverse_width_4(word: u64) -> u64 {
    let word = ((word & 0xf0f0_f0f0_f0f0_f0f0) >> 4) | ((word & 0x0f0f_0f0f_0f0f_0f0f) << 4);

    word.swap_bytes()
}

#[inline(always)]
const fn reverse_width_16(word: u64) -> u64 {
    let word = ((word & 0xffff_0000_ffff_0000) >> 16) | ((word & 0x0000_ffff_0000_ffff) << 16);

    word.rotate_left(32)
}

#[inline(always)]
pub(super) const fn reverse_full_words(words: &mut [u64], active_words: usize, width: usize) {
    let mut left = 0;
    let mut right = active_words - 1;

    while left < right {
        let lhs = reverse_word(words[left], width);
        let rhs = reverse_word(words[right], width);

        words[left] = rhs;
        words[right] = lhs;
        left += 1;
        right -= 1;
    }

    if left == right {
        words[left] = reverse_word(words[left], width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_word_helpers_are_const_capable() {
        const WIDTH_2: u64 = reverse_word(0b00_01_10_11u64 << 56, 2);
        const WIDTH_8: u64 = reverse_word(0x00_01_02_03_04_05_06_07, 8);

        assert_eq!(WIDTH_2, 0b11_10_01_00);
        assert_eq!(WIDTH_8, 0x07_06_05_04_03_02_01_00);
    }
}
