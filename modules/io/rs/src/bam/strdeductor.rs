use noodles::bam::Record;

use biobit_core_rs::loc::Orientation;

pub trait StrDeductor: Send + Sync + Clone {
    fn deduce(&mut self, records: &Record) -> Orientation;
}

impl<T> StrDeductor for T
where
    T: FnMut(&Record) -> Orientation + Clone + Send + Sync,
{
    fn deduce(&mut self, records: &Record) -> Orientation {
        (self)(records)
    }
}

pub mod deduce {
    use super::*;

    pub mod se {
        use super::*;

        #[inline(always)]
        pub fn unstranded(_: &Record) -> Orientation {
            Orientation::Dual
        }

        #[inline(always)]
        pub fn forward(record: &Record) -> Orientation {
            _forward(record.flags().is_reverse_complemented())
        }

        #[inline(always)]
        pub fn _forward(is_reverse: bool) -> Orientation {
            if is_reverse {
                Orientation::Reverse
            } else {
                Orientation::Forward
            }
        }

        #[inline(always)]
        pub fn reverse(record: &Record) -> Orientation {
            _reverse(record.flags().is_reverse_complemented())
        }
        #[inline(always)]
        pub fn _reverse(is_reverse: bool) -> Orientation {
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
        pub fn unstranded(_: &Record) -> Orientation {
            Orientation::Dual
        }

        #[inline(always)]
        pub fn forward(record: &Record) -> Orientation {
            _forward(
                record.flags().is_first_segment(),
                record.flags().is_reverse_complemented(),
            )
        }

        #[inline(always)]
        pub fn _forward(is_first: bool, is_reverse: bool) -> Orientation {
            match (is_first, is_reverse) {
                (true, true) => Orientation::Reverse,
                (true, false) => Orientation::Forward,
                (false, true) => Orientation::Forward,
                (false, false) => Orientation::Reverse,
            }
        }

        #[inline(always)]
        pub fn reverse(record: &Record) -> Orientation {
            _reverse(
                record.flags().is_first_segment(),
                record.flags().is_reverse_complemented(),
            )
        }

        #[inline(always)]
        pub fn _reverse(is_first: bool, is_reverse: bool) -> Orientation {
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
        assert_eq!(deduce::se::_reverse(true), Orientation::Forward);
        assert_eq!(deduce::se::_reverse(false), Orientation::Reverse);

        assert_eq!(deduce::se::_forward(true), Orientation::Reverse);
        assert_eq!(deduce::se::_forward(false), Orientation::Forward);
    }

    #[test]
    fn test_paired_end_library() {
        assert_eq!(deduce::pe::_reverse(true, true), Orientation::Forward);
        assert_eq!(deduce::pe::_reverse(true, false), Orientation::Reverse);
        assert_eq!(deduce::pe::_reverse(false, true), Orientation::Reverse);
        assert_eq!(deduce::pe::_reverse(false, false), Orientation::Forward);

        assert_eq!(deduce::pe::_forward(true, true), Orientation::Reverse);
        assert_eq!(deduce::pe::_forward(true, false), Orientation::Forward);
        assert_eq!(deduce::pe::_forward(false, true), Orientation::Forward);
        assert_eq!(deduce::pe::_forward(false, false), Orientation::Reverse);
    }
}
