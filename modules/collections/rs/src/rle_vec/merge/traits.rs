use ::impl_tools::autoimpl;

#[autoimpl(for <M: trait + ?Sized> &mut M, Box<M>)]
pub trait TryMerge<T> {
    type Error;

    fn first(&mut self, value: &T) -> Result<T, Self::Error>;

    #[inline(always)]
    fn second(&mut self, value: &T) -> Result<T, Self::Error> {
        self.first(value)
    }

    fn both(&mut self, first: &T, second: &T) -> Result<T, Self::Error>;
}

impl<T, E, Single, Both> TryMerge<T> for (Single, Both)
where
    Single: FnMut(&T) -> Result<T, E>,
    Both: FnMut(&T, &T) -> Result<T, E>,
{
    type Error = E;

    #[inline(always)]
    fn first(&mut self, value: &T) -> Result<T, Self::Error> {
        (self.0)(value)
    }

    #[inline(always)]
    fn both(&mut self, first: &T, second: &T) -> Result<T, Self::Error> {
        (self.1)(first, second)
    }
}

impl<T, E, First, Second, Both> TryMerge<T> for (First, Second, Both)
where
    First: FnMut(&T) -> Result<T, E>,
    Second: FnMut(&T) -> Result<T, E>,
    Both: FnMut(&T, &T) -> Result<T, E>,
{
    type Error = E;

    #[inline(always)]
    fn first(&mut self, value: &T) -> Result<T, Self::Error> {
        (self.0)(value)
    }

    #[inline(always)]
    fn second(&mut self, value: &T) -> Result<T, Self::Error> {
        (self.1)(value)
    }

    #[inline(always)]
    fn both(&mut self, first: &T, second: &T) -> Result<T, Self::Error> {
        (self.2)(first, second)
    }
}
