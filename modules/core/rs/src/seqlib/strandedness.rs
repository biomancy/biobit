use std::fmt::Display;

use crate::loc::Orientation;

/// Strandedness of a sequencing library. The meaning of each variant depends on the library type.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(i8)]
pub enum Strandedness {
    Forward = 1,
    Reverse = -1,
    Unstranded = 0,
}

impl Display for Strandedness {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Strandedness::Forward => write!(f, "F"),
            Strandedness::Reverse => write!(f, "R"),
            Strandedness::Unstranded => write!(f, "U"),
        }
    }
}

pub mod deduce {
    use super::*;

    pub mod se {
        use super::*;

        #[inline(always)]
        pub fn forward(is_reverse: bool) -> Orientation {
            if is_reverse {
                Orientation::Reverse
            } else {
                Orientation::Forward
            }
        }

        #[inline(always)]
        pub fn reverse(is_reverse: bool) -> Orientation {
            if is_reverse {
                Orientation::Forward
            } else {
                Orientation::Reverse
            }
        }
    }
    pub mod pe {
        use super::*;

        #[inline(always)]
        pub fn forward(is_first: bool, is_reverse: bool) -> Orientation {
            match (is_first, is_reverse) {
                (true, true) => Orientation::Reverse,
                (true, false) => Orientation::Forward,
                (false, true) => Orientation::Forward,
                (false, false) => Orientation::Reverse,
            }
        }

        #[inline(always)]
        pub fn reverse(is_first: bool, is_reverse: bool) -> Orientation {
            match (is_first, is_reverse) {
                (true, true) => Orientation::Forward,
                (true, false) => Orientation::Reverse,
                (false, true) => Orientation::Reverse,
                (false, false) => Orientation::Forward,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_end_library() {
        assert_eq!(deduce::se::reverse(true), Orientation::Forward);
        assert_eq!(deduce::se::reverse(false), Orientation::Reverse);

        assert_eq!(deduce::se::forward(true), Orientation::Reverse);
        assert_eq!(deduce::se::forward(false), Orientation::Forward);
    }

    #[test]
    fn test_paired_end_library() {
        assert_eq!(deduce::pe::reverse(true, true), Orientation::Forward);
        assert_eq!(deduce::pe::reverse(true, false), Orientation::Reverse);
        assert_eq!(deduce::pe::reverse(false, true), Orientation::Reverse);
        assert_eq!(deduce::pe::reverse(false, false), Orientation::Forward);

        assert_eq!(deduce::pe::forward(true, true), Orientation::Reverse);
        assert_eq!(deduce::pe::forward(true, false), Orientation::Forward);
        assert_eq!(deduce::pe::forward(false, true), Orientation::Forward);
        assert_eq!(deduce::pe::forward(false, false), Orientation::Reverse);
    }
}
