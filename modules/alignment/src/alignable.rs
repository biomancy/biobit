// Instead of making a custom trait here I must support Rust builtin traits for containers
// once they are ready: https://internals.rust-lang.org/t/traits-that-should-be-in-std-but-arent/3002
pub trait Alignable {
    type Symbol;

    fn len(&self) -> usize;
    fn at(&self, pos: usize) -> &Self::Symbol;
}

impl<'a, T: Copy> Alignable for &'a [T] {
    type Symbol = T;

    #[inline(always)]
    fn len(&self) -> usize { (self as &[Self::Symbol]).len() }

    #[inline(always)]
    fn at(&self, pos: usize) -> &Self::Symbol { &self[pos] }
}

pub struct Reversed<T: Alignable> {
    base: T,
}

impl<T: Alignable> Reversed<T> {
    pub fn new(alignable: T) -> Self {
        Self { base: alignable }
    }
}

impl<T: Alignable> Alignable for Reversed<T> {
    type Symbol = T::Symbol;

    #[inline(always)]
    fn len(&self) -> usize {
        self.base.len()
    }

    #[inline(always)]
    fn at(&self, pos: usize) -> &Self::Symbol {
        self.base.at(self.base.len() - pos - 1)
    }
}
