use std::collections::TryReserveError;
use std::ops::RangeBounds;

use num::{PrimInt, Unsigned};

/// A trait representing the concept of identity for run-length encoding.
///
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
    fn identical(&mut self, first: &T, second: &T) -> bool;
}

/// An implementation of the `Identical` trait for any function that takes two references to `T` and returns a `bool`.
impl<T, F> Identical<T> for F where F: FnMut(&T, &T) -> bool {
    #[inline]
    fn identical(&mut self, first: &T, second: &T) -> bool {
        self(first, second)
    }
}

/// A trait representing the position type used to store endpoints of runs in run-length encoded vector.
///
/// This trait is designed to allow the use of large integers for the endpoints without being limited by the platform's pointer size (i.e., `usize`).
pub trait Position: PrimInt {}

/// An implementation of the `Position` trait for any type `T` that is a primitive integer.
impl<T: PrimInt> Position for T {}

/// A trait representing the length type used to store the lengths of runs in run-length encoded vector.
///
/// This is beneficial to use lower precision for lengths than for indices, as the lengths can be generally expected to be smaller.
pub trait Length: PrimInt + Unsigned {}

/// An implementation of the `RleLength` trait for any type `T` that is a primitive unsigned integer.
impl<T: PrimInt + Unsigned> Length for T {}


pub enum Repack<T> {
    KeepFirst,
    KeepSecond,
    KeepBoth,
    Merge(T),
}


trait RxeVecCommons<T> {
    fn append(&mut self, other: &mut Self);
    fn capacity(&self) -> usize;
    fn truncate(&mut self, len: usize);
    fn reserve(&mut self, additional: usize);
    fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError>;
    fn reserve_exact(&mut self, additional: usize);
    fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError>;
    fn shrink_to(&mut self, min_capacity: usize);
    fn shrink_to_fit(&mut self);
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    unsafe fn set_len(&mut self, new_len: usize);
    fn repack(&mut self);
    fn repack_by<F>(&mut self, func: F) where F: FnMut(&mut T, &mut T) -> Repack<T>;
    fn extend_from_within(&mut self, src: impl RangeBounds<usize>);
    fn copy_from_within(&mut self, src: impl RangeBounds<usize>, dst: usize);
    fn swap(&mut self, a: usize, b: usize);
    unsafe fn swap_unchecked(&mut self, a: usize, b: usize);
    fn swap_with(&mut self, other: &mut Self);
    fn retain<F>(&mut self, func: F) where F: FnMut(&T) -> bool;
    fn retain_mut<F>(&mut self, func: F) where F: FnMut(&mut T) -> bool;
    fn split_off(&mut self, at: usize) -> Self;
    fn repeat(&mut self, n: usize);
    fn reverse(&mut self);
    fn rotate_left(&mut self, n: usize);
    fn rotate_right(&mut self, n: usize);

    // For this trait
    // -----------------------
    //  binary_search
    //  binary_search_by
    //  binary_search_by_key
    //  contains
    //  fill
    //  fill_with
    //


    // For individual vectors
    // -----------------------
    // drain


    // For individual views
    // -----------------------
    // insert
    // starts_with
    // swap_remove
    // extract_if
    // insert
    // pop
    // push
    // push_within_capacity
    // remove
    // dissolve
    // resize
    // resize_with
    // spare_capacity_mut
    // splice
    // split_at_spare_mut
    // extend_from_slice
    // clone_from_slice
    // copy_from_slice
    // first
    // first_mut
    // last
    // last_mut
    // iter
    // iter_mut
    // partition_point
    // rsplit
    // rsplit_mut
    // rsplitn
    // rsplitn_mut
    // select_nth_unstable
    // select_nth_unstable_by
    // select_nth_unstable_by_key
    // sort
    // sort_by
    // sort_by_cached_key
    // sort_by_key
    // sort_floats
    // sort_floats
    // sort_unstable
    // sort_unstable_by
    // sort_unstable_by_key
    // windows

    // Unclear
    // -----------------------
    // split
    // split_at
    // split_at_mut
    // split_at_mut_unchecked
    // split_at_unchecked
    // split_first
    // split_first_chunk
    // split_first_chunk_mut
    // split_first_mut
    // split_inclusive
    // split_inclusive_mut
    // split_last
    // split_last_chunk
    // split_last_chunk_mut
    // split_last_mut
    // split_mut
    // split_once
    // splitn
    // splitn_mut
    // starts_with
    // strip_prefix
    // strip_suffix
}

// Trait Implementations
//     Clone
//     Debug
//     Default
//     Deref
//     DerefMut
//     Drop
//     Eq
//     Hash
//     Ord
// ------------------------------
//     Extend<&'a T>
//     Extend<T>
// ------------------------------
//     From<&'a Vec<T>>
//     From<&[T; N]>
//     From<&[T]>
//     From<&mut [T; N]>
//     From<&mut [T]>
//     From<&str>
//     From<BinaryHeap<T, A>>
//     From<Box<[T], A>>
//     From<CString>
//     From<Cow<'a, [T]>>
//     From<String>
//     From<Vec<NonZeroU8>>
//     From<Vec<T, A>>
//     From<Vec<T, A>>
//     From<Vec<T, A>>
//     From<Vec<T, A>>
//     From<Vec<T, A>>
//     From<Vec<T>>
//     From<VecDeque<T, A>>
//     From<[T; N]>
//     FromIterator<T>
// ------------------------------
//     Index<I>
//     IndexMut<I>
//     IntoIterator
// ------------------------------
//     PartialEq<&[U; N]>
//     PartialEq<&[U]>
//     PartialEq<&mut [U]>
//     PartialEq<Vec<U, A2>>
//     PartialEq<Vec<U, A>>
//     PartialEq<Vec<U, A>>
//     PartialEq<Vec<U, A>>
//     PartialEq<Vec<U, A>>
//     PartialEq<Vec<U, A>>
//     PartialEq<[U; N]>
//     PartialEq<[U]>
//     PartialOrd<Vec<T, A2>>
// ------------------------------
//     Write
