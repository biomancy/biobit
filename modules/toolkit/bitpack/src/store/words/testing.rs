pub(crate) use super::utils::low_mask;

pub(crate) fn words_from_scalars<const N: usize>(width: usize, scalars: &[u64]) -> [u64; N] {
    let mut words = [0; N];

    for (pos, scalar) in scalars.iter().copied().enumerate() {
        write_scalar(&mut words, width, pos, scalar);
    }

    words
}

pub(crate) fn get_reference(words: &[u64], width: usize, pos: usize) -> u64 {
    let mut scalar = 0;

    for bit in 0..width {
        scalar <<= 1;

        if read_bit(words, (pos * width) + bit) {
            scalar |= 1;
        }
    }

    scalar
}

pub(crate) fn set_reference(words: &mut [u64], width: usize, pos: usize, scalar: u64) {
    let scalar = scalar & low_mask(width);

    for bit in 0..width {
        let source_bit = width - bit - 1;
        let target_bit = (pos * width) + bit;

        if ((scalar >> source_bit) & 1) == 0 {
            clear_bit(words, target_bit);
        } else {
            set_bit(words, target_bit);
        }
    }
}

pub(crate) fn reverse_reference<const N: usize>(
    words: &[u64; N],
    width: usize,
    len: usize,
) -> [u64; N] {
    let mut reversed = [0; N];

    for pos in 0..len {
        let scalar = get_reference(words, width, pos);
        set_reference(&mut reversed, width, len - pos - 1, scalar);
    }

    reversed
}

fn write_scalar(words: &mut [u64], width: usize, pos: usize, scalar: u64) {
    let scalar = scalar & low_mask(width);

    for bit in 0..width {
        let source_bit = width - bit - 1;

        if ((scalar >> source_bit) & 1) != 0 {
            set_bit(words, (pos * width) + bit);
        }
    }
}

fn read_bit(words: &[u64], bit: usize) -> bool {
    let word = bit / u64::BITS as usize;
    let offset = bit % u64::BITS as usize;

    ((words[word] >> (u64::BITS as usize - offset - 1)) & 1) != 0
}

fn set_bit(words: &mut [u64], bit: usize) {
    let word = bit / u64::BITS as usize;
    let offset = bit % u64::BITS as usize;

    words[word] |= 1 << (u64::BITS as usize - offset - 1);
}

fn clear_bit(words: &mut [u64], bit: usize) {
    let word = bit / u64::BITS as usize;
    let offset = bit % u64::BITS as usize;

    words[word] &= !(1 << (u64::BITS as usize - offset - 1));
}
