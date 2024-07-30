/// A trait representing the concept of identity for run-length encoding.
/// This trait is used to determine whether two values should be considered as part of the same run.
pub trait Identical<T> {
    /// Determines if two values are identical for the purpose of run-length encoding.
    ///
    /// # Arguments
    ///
    /// * `first` - The first value to compare.
    /// * `second` - The second value to compare.
    ///
    /// # Returns
    ///
    /// Returns `true` if the two values should be considered part of the same run.
    /// Note that no guarantees are made about which of the two values will be kept in the run.
    fn identical(&self, first: &T, second: &T) -> bool;
}

/// An implementation of the `Identical` trait for any function that takes two references to `T` and returns a `bool`.
impl<T, F> Identical<T> for F
where
    F: Fn(&T, &T) -> bool,
{
    #[inline]
    fn identical(&self, first: &T, second: &T) -> bool {
        self(first, second)
    }
}
