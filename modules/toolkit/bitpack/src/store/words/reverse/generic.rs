use super::super::utils;

#[inline(always)]
pub(super) const fn reverse_fields(words: &mut [u64], width: usize, len: usize) {
    let mask = utils::low_mask(width);
    let mut left = 0;
    let mut right = len - 1;

    while left < right {
        let lhs = utils::get_unchecked(words, width, left) & mask;
        let rhs = utils::get_unchecked(words, width, right) & mask;

        utils::set_unchecked(words, width, left, rhs);
        utils::set_unchecked(words, width, right, lhs);
        left += 1;
        right -= 1;
    }
}
