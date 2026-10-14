use super::{ErasedMapping, Fallible, Infallible, Selector, Then};
use crate::access::private;
use crate::access::{Access, DynAccess, DynMapping, DynStore, Kind, Read, Result, Session, Store};

/// A known typed store and a concrete selector. No lookup path is retained.
#[derive(Clone)]
pub struct StaticMapping<R, S> {
    root: R,
    selector: S,
}

impl<R, S> StaticMapping<R, S> {
    /// Creates a mapping from a store and a borrowed selector.
    /// Closure-based construction is available through [`Store::map`] and
    /// [`Store::try_map`].
    pub fn new(root: R, selector: S) -> Self
    where
        R: Store,
        S: Selector<R::Target>,
    {
        Self { root, selector }
    }

    pub fn root(&self) -> &R {
        &self.root
    }
}

impl<R, S> StaticMapping<R, S>
where
    R: Store + Sync,
    S: Selector<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    /// Composes another selector over the original store without allocating.
    /// The result is clonable when the root and both selectors are clonable.
    pub fn map<U: ?Sized + 'static, F>(self, next: F) -> StaticMapping<R, Then<S, Infallible<F>>>
    where
        F: for<'a> Fn(&'a S::Target) -> &'a U + Send + Sync + 'static,
    {
        StaticMapping::new(self.root, Then::new(self.selector, Infallible::new(next)))
    }

    /// Composes a fallible selector over the original store without allocating.
    pub fn try_map<U: ?Sized + 'static, F>(self, next: F) -> StaticMapping<R, Then<S, Fallible<F>>>
    where
        F: for<'a> Fn(&'a S::Target) -> Result<&'a U> + Send + Sync + 'static,
    {
        StaticMapping::new(self.root, Then::new(self.selector, Fallible::new(next)))
    }

    /// Shares this mapping in an Arc, erasing the root and selector types.
    pub fn erase(self) -> ErasedMapping<S::Target> {
        ErasedMapping::new(self)
    }
}

impl<R, S> private::Sealed for StaticMapping<R, S> {}

impl<R, S> DynAccess for StaticMapping<R, S>
where
    R: Store + Sync,
    S: Selector<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn kind(&self) -> Kind<'_> {
        Kind::Mapping(self)
    }
}

impl<R, S> Access for StaticMapping<R, S>
where
    R: Store + Sync,
    S: Selector<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    type Target = S::Target;

    fn borrow<'a>(&'a self, session: &'a Session) -> Result<Read<'a, Self::Target>> {
        self.root
            .borrow(session)?
            .try_map(|value| self.selector.select(value))
    }
}

impl<R, S> DynMapping for StaticMapping<R, S>
where
    R: Store + Sync,
    S: Selector<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn root(&self) -> &dyn DynStore {
        &self.root
    }
}
