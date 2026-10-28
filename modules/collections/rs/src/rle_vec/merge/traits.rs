use ::impl_tools::autoimpl;

/// Combines input values, optionally producing a different output type.
#[autoimpl(for <M: trait + ?Sized> &mut M, Box<M>)]
pub trait TryMerge<T, Output> {
    type Error;

    fn first(&mut self, value: &T) -> Result<Output, Self::Error>;

    #[inline(always)]
    fn second(&mut self, value: &T) -> Result<Output, Self::Error> {
        self.first(value)
    }

    fn both(&mut self, first: &T, second: &T) -> Result<Output, Self::Error>;
}

impl<T, Output, E, Single, Both> TryMerge<T, Output> for (Single, Both)
where
    Single: FnMut(&T) -> Result<Output, E>,
    Both: FnMut(&T, &T) -> Result<Output, E>,
{
    type Error = E;

    #[inline(always)]
    fn first(&mut self, value: &T) -> Result<Output, Self::Error> {
        (self.0)(value)
    }

    #[inline(always)]
    fn both(&mut self, first: &T, second: &T) -> Result<Output, Self::Error> {
        (self.1)(first, second)
    }
}

impl<T, Output, E, First, Second, Both> TryMerge<T, Output> for (First, Second, Both)
where
    First: FnMut(&T) -> Result<Output, E>,
    Second: FnMut(&T) -> Result<Output, E>,
    Both: FnMut(&T, &T) -> Result<Output, E>,
{
    type Error = E;

    #[inline(always)]
    fn first(&mut self, value: &T) -> Result<Output, Self::Error> {
        (self.0)(value)
    }

    #[inline(always)]
    fn second(&mut self, value: &T) -> Result<Output, Self::Error> {
        (self.1)(value)
    }

    #[inline(always)]
    fn both(&mut self, first: &T, second: &T) -> Result<Output, Self::Error> {
        (self.2)(first, second)
    }
}
