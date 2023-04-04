pub use super::Symbol;

// Instead of making a custom trait here I must support Rust builtin traits for containers
// once they are ready: https://internals.rust-lang.org/t/traits-that-should-be-in-std-but-arent/3002
pub trait Alignable {
    fn len(&self) -> usize;
    fn at(&self, pos: usize) -> Symbol;
}

impl<'a> Alignable for &'a [Symbol] {
    #[inline(always)]
    fn len(&self) -> usize {
        (&self as &[Symbol]).len()
    }

    #[inline(always)]
    fn at(&self, pos: usize) -> Symbol {
        self[pos]
    }
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
    #[inline(always)]
    fn len(&self) -> usize {
        self.base.len()
    }

    #[inline(always)]
    fn at(&self, pos: usize) -> Symbol {
        self.base.at(self.base.len() - pos - 1)
    }
}
