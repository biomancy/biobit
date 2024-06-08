pub struct RpeVec {}


pub struct RpeVecBuilder {}

// Design and implement a data structure named RleVec using the programming language Rust. This data structure must run-length encode stored data while also supporting the following features:
//
// House data as two vectors: one for the values and another for run boundaries.
// Enable parameterized element type (any type) and boundary type (only unsigned integers).
// Enable a parameterized equality function, which is used to compare two adjacent elements and determine if they can be classified in the same run. By default, it should support standard equality.
// Enable a merge function that can combine two RleVecs into one. This function should be parameterized to allow a custom merge function per run. In other words, this function will be used to merge values of two overlapping runs into one.
// Support indexing that gives the element at the specified index as though the RleVec were a conventional Vec. That is, the index should be the actual - not the Rle - index.
// Allow for RleVec optimization through a user-defined function to merge similar elements (repack). If the function results in None, then the elements are not merged. Conversely, a Some outcome indicates a merged value.
// Enable iteration over the RleVec which returns a Run structure that offers references to the value and length of the run. This should be available in both mutable and non-mutable versions.
// Support iteration over the runs and run indices.
// Support standard iteration where each element is returned individually, the same as in traditional Vec.
// Implement any other methods you believe are helpful for this data structure, particularly those that align with standard Vec capabilities.
// Develop tests for the RleVec to ensure it functions as anticipated.

// Methods to implement:
// append
// as_mut_ptr
// as_mut_slice
// as_ptr
// as_slice
// capacity
// clear
// dedup
// dedup_by
// dedup_by_key
// drain
// extend_from_slice
// extend_from_within
// extract_if
// from_raw_parts
// from_raw_parts_in
// insert
// into_boxed_slice
// into_flattened
// into_raw_parts
// into_raw_parts_with_alloc
// is_empty
// leak
// len
// new
// new_in
// pop
// push
// push_within_capacity
// remove
// reserve
// reserve_exact
// resize
// resize_with
// retain
// retain_mut
// set_len
// shrink_to
// shrink_to_fit
// spare_capacity_mut
// splice
// split_at_spare_mut
// split_off
// swap_remove
// truncate
// try_reserve
// try_reserve_exact
// with_capacity
// with_capacity_in

// Traits to implement
//     AsMut<Vec<T, A>>
//     AsMut<[T]>
//     AsRef<Vec<T, A>>
//     AsRef<[T]>
//     Borrow<[T]>
//     BorrowMut<[T]>
//     Clone
//     Debug
//     Default
//     Deref
//     DerefMut
//     Drop
//     Eq
//     Extend<&'a T>
//     Extend<T>
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
//     Hash
//     Index<I>
//     IndexMut<I>
//     IntoIterator
//     IntoIterator
//     IntoIterator
//     Ord
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
//     TryFrom<Vec<T, A>>
//     TryFrom<Vec<T>>
//     Write
//     From<RleVec<T, Merge, Ind>> for Vec<T>


//
// #[inline]
// pub fn is_empty(&self) -> bool {
//     self.endpoints.is_empty()
// }

// #[inline]
// pub fn clear(&mut self) {
//     self.endpoints.clear();
//     self.values.clear();
// }

// #[inline]
// pub fn items(&self) -> Ind {
//     self.endpoints.last().cloned().unwrap_or(Ind::zero())
// }

// #[inline]
// pub fn capacity(&self) -> usize {
//     debug_assert!(self.endpoints.capacity() == self.values.capacity());
//     self.values.capacity()
// }

// pub fn reserve(&mut self, additional_runs: usize) {
//     self.endpoints.reserve(additional_runs);
//     self.values.reserve(additional_runs);
// }

// pub fn reserve_exact(&mut self, additional_runs: usize) {
//     self.endpoints.reserve_exact(additional_runs);
//     self.values.reserve_exact(additional_runs);
// }

// pub fn try_reserve(&mut self, additional_runs: usize) -> Result<(), TryReserveError> {
//     self.endpoints.try_reserve(additional_runs)?;
//     self.values.try_reserve(additional_runs)
// }

// pub fn try_reserve_exact(&mut self, additional_runs: usize) -> Result<(), TryReserveError> {
//     self.endpoints.try_reserve_exact(additional_runs)?;
//     self.values.try_reserve_exact(additional_runs)
// }

// pub fn shrink_to(&mut self, min_runs: usize) {
//     self.endpoints.shrink_to(min_runs);
//     self.values.shrink_to(min_runs);
// }

// pub fn shrink_to_fit(&mut self) {
//     self.endpoints.shrink_to_fit();
//     self.values.shrink_to_fit();
// }

// pub fn truncate(&mut self, runs: usize) {
//     self.endpoints.truncate(runs);
//     self.values.truncate(runs);
// }

// pub fn resize(&mut self, new_len: usize, value: T, length: Ind)
//     where T: Clone
// {
//     self.endpoints.resize(new_len, length);
//     self.values.resize(new_len, value);
//     todo!("Update endpoints")
// }

// pub fn resize_with(&mut self, new_len: usize, f: impl FnMut() -> (T, Ind)) {
//     let len = self.len();
//     if new_len > len {
//         iter::repeat_with(f).take(new_len - len).for_each(|(value, length)| {
//             self.push(value, length);
//         });
//     } else {
//         self.truncate(new_len);
//     }
// }

// #[inline]
// pub fn insert(&mut self, index: Ind, value: T, count: Ind) {
//     // self.endpoints.insert()
//
//     // Find the run that contains the index
//     let index = self.endpoints.binary_search(&index).unwrap_or_else(|x| x);
//
//
//     todo!()
// }

// #[inline]
// pub fn pop(&mut self) -> Option<(Ind, T)> {
//     debug_assert!(self.endpoints.len() == self.values.len());
//     match (self.endpoints.pop(), self.values.pop()) {
//         (Some(x), Some(y)) => Some((x, y)),
//         _ => None,
//     }
// }

// pub fn push(&mut self, value: T, count: Ind) {
//     if self.is_empty() {
//         self.values.push(value);
//         self.endpoints.push(count);
//         return;
//     }
//
//     if self.identical.identical(self.values.last().unwrap(), &value) {
//         let endpoint = self.endpoints.last_mut().unwrap();
//         *endpoint = *endpoint + count;
//     } else {
//         self.endpoints.push(*self.endpoints.last().unwrap() + count);
//         self.values.push(value);
//     }
// }

// pub fn remove(&mut self, index: Ind) -> T {
//     todo!()
// }

// pub fn retain_mut(&mut self, mut f: impl FnMut(&mut T) -> bool) {
//     let mut current = 0;
//     let mut next = 1;
//     while next < self.endpoints.len() {
//         if f(&mut self.values[current]) {
//             current += 1;
//         } else {
//             self.values.swap(current, next);
//             self.endpoints[current] = self.endpoints[next];
//         }
//         next += 1;
//     }
//     self.values.truncate(current + 1);
//     self.endpoints.truncate(current + 1);
// }

// pub fn retain(&mut self, mut f: impl FnMut(&T) -> bool) {
//     self.retain_mut(|x| f(x));
// }

// pub fn split_off(&mut self, at: Ind) -> Self
//     where M: Clone
// {
//     todo!()
//     // let other_endpoints = self.endpoints.split_off(at);
//     // let other_values = self.values.split_off(at);
//     // Self {
//     //     endpoints: other_endpoints,
//     //     values: other_values,
//     //     merge: self.merge.clone(),
//     // }
// }
//
// pub fn swap_remove(&mut self, index: Ind) -> T {
//     todo!()
// }
