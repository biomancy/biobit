// #[repr(transparent)]
// pub struct MemoryPlaceholder<T>(T);
//
// unsafe impl<T> Send for MemoryPlaceholder<T> {}
// unsafe impl<T> Sync for MemoryPlaceholder<T> {}
//
// pub trait StashType {
//     type Stashed: Send + Sync + 'rigid;
// }
//
// pub trait Stash: StashType + Send + Sync + 'rigid {
//     fn stash(self) -> Self::Stashed;
//
//     fn unstash(source: Self::Stashed) -> Self;
// }
//
// impl<T: 'rigid> StashType for &T {
//     type Stashed = MemoryPlaceholder<&'rigid T>;
// }
//
// impl<T: StashType> StashType for Vec<T> {
//     type Stashed = Vec<T::Stashed>;
// }
//
// impl StashType for i32 {
//     type Stashed = i32;
// }
//
// impl Stash for i32 {
//     fn stash(self) -> Self::Stashed {
//         self
//     }
//
//     fn unstash(source: Self::Stashed) -> Self {
//         source
//     }
// }
//
// impl<T: Stash> Stash for Vec<T> {
//     fn stash(mut self) -> Self::Stashed {
//         self.clear();
//         self.into_iter().map(|x| unreachable!()).collect()
//     }
//
//     fn unstash(source: Self::Stashed) -> Self {
//         source.into_iter().map(|x| unreachable!()).collect()
//     }
// }
//
// fn tmp() {
//     let a = vec![1, 2, 3];
//     let b = a.stash();
//     let c = Vec::<i32>::unstash(b);
// }
