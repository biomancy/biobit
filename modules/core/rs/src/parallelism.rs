use std::cmp::Ordering;
use std::thread::available_parallelism;

use eyre::Result;

fn _normalize(requested: isize, max: isize) -> usize {
    match requested.cmp(&0) {
        Ordering::Less => (max + requested + 1).max(1) as usize,
        Ordering::Equal => 1,
        Ordering::Greater => requested.min(max) as usize,
    }
}

pub fn available(requested: isize) -> Result<usize> {
    let max = available_parallelism()?.get() as isize;
    Ok(_normalize(requested, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallelism_normalization() {
        for (threads, max, expected) in [
            (0, 4, 1),
            (1, 4, 1),
            (2, 4, 2),
            (3, 4, 3),
            (4, 4, 4),
            (5, 4, 4),
            (1231, 4, 4),
            (-1, 4, 4),
            (-2, 4, 3),
            (-3, 4, 2),
            (-4, 4, 1),
            (-5, 4, 1),
        ] {
            assert_eq!(_normalize(threads, max), expected);
        }
    }
}
