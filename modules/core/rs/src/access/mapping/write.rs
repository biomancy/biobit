use super::{ErasedMappingMut, Fallible, Infallible, SelectorMut, StaticMapping, Then};
use crate::access::private;
use crate::access::{
    Access, AccessMut, DynAccess, DynAccessMut, DynMapping, DynMappingMut, DynStore, DynStoreMut,
    Kind, KindMut, Read, Result, Session, StoreMut, Write,
};

/// A concrete field selection borrowed exclusively under its store's lease.
/// Borrow conflicts cover the whole store, including disjoint fields.
#[derive(Clone)]
pub struct StaticMappingMut<R, S> {
    root: R,
    selector: S,
}

impl<R, S> StaticMappingMut<R, S> {
    /// Creates a mapping from a store and a shared/mutable selector.
    /// Closure-based construction is available through [`StoreMut::map_mut`] and
    /// [`StoreMut::try_map_mut`].
    pub fn new(root: R, selector: S) -> Self
    where
        R: StoreMut,
        S: SelectorMut<R::Target>,
    {
        Self { root, selector }
    }

    pub fn root(&self) -> &R {
        &self.root
    }

    pub fn root_mut(&self) -> &R {
        &self.root
    }
}

impl<R, S> StaticMappingMut<R, S>
where
    R: StoreMut + Sync,
    S: SelectorMut<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    /// Composes a shared selector over the original store without allocating.
    pub fn map<U: ?Sized + 'static, F>(self, next: F) -> StaticMapping<R, Then<S, Infallible<F>>>
    where
        F: for<'a> Fn(&'a S::Target) -> &'a U + Send + Sync + 'static,
    {
        self.into_read_only().map(next)
    }

    /// Composes a fallible shared selector over the original store.
    pub fn try_map<U: ?Sized + 'static, F>(self, next: F) -> StaticMapping<R, Then<S, Fallible<F>>>
    where
        F: for<'a> Fn(&'a S::Target) -> Result<&'a U> + Send + Sync + 'static,
    {
        self.into_read_only().try_map(next)
    }

    /// Composes shared and mutable selectors over the original store.
    pub fn map_mut<U: ?Sized + 'static, FR, FW>(
        self,
        read: FR,
        write: FW,
    ) -> StaticMappingMut<R, Then<S, Infallible<FR, FW>>>
    where
        FR: for<'a> Fn(&'a S::Target) -> &'a U + Send + Sync + 'static,
        FW: for<'a> Fn(&'a mut S::Target) -> &'a mut U + Send + Sync + 'static,
    {
        StaticMappingMut::new(
            self.root,
            Then::new(self.selector, Infallible::new_mut(read, write)),
        )
    }

    /// Composes fallible shared and mutable selectors without allocating.
    pub fn try_map_mut<U: ?Sized + 'static, FR, FW>(
        self,
        read: FR,
        write: FW,
    ) -> StaticMappingMut<R, Then<S, Fallible<FR, FW>>>
    where
        FR: for<'a> Fn(&'a S::Target) -> Result<&'a U> + Send + Sync + 'static,
        FW: for<'a> Fn(&'a mut S::Target) -> Result<&'a mut U> + Send + Sync + 'static,
    {
        StaticMappingMut::new(
            self.root,
            Then::new(self.selector, Fallible::new_mut(read, write)),
        )
    }

    /// Shares this mapping in an Arc, erasing the root and selector types.
    pub fn erase(self) -> ErasedMappingMut<S::Target> {
        ErasedMappingMut::new(self)
    }

    /// Clones the root and selector into a view with only read capability.
    pub fn read_only(&self) -> StaticMapping<R, S>
    where
        S: Clone,
    {
        StaticMapping::new(self.root.clone(), self.selector.clone())
    }

    /// Transfers the root and selector into a read-only view without cloning.
    pub fn into_read_only(self) -> StaticMapping<R, S> {
        StaticMapping::new(self.root, self.selector)
    }
}

impl<R, S> private::Sealed for StaticMappingMut<R, S> {}

impl<R, S> DynAccess for StaticMappingMut<R, S>
where
    R: StoreMut + Sync,
    S: SelectorMut<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn kind(&self) -> Kind<'_> {
        Kind::Mapping(self)
    }
}

impl<R, S> DynAccessMut for StaticMappingMut<R, S>
where
    R: StoreMut + Sync,
    S: SelectorMut<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn kind_mut(&self) -> KindMut<'_> {
        KindMut::Mapping(self)
    }
}

impl<R, S> Access for StaticMappingMut<R, S>
where
    R: StoreMut + Sync,
    S: SelectorMut<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    type Target = S::Target;

    fn borrow<'a>(&'a self, session: &'a Session) -> Result<Read<'a, Self::Target>> {
        self.root
            .borrow(session)?
            .try_map(|value| self.selector.select(value))
    }
}

impl<R, S> AccessMut for StaticMappingMut<R, S>
where
    R: StoreMut + Sync,
    S: SelectorMut<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn borrow_mut<'a>(&'a self, session: &'a Session) -> Result<Write<'a, Self::Target>> {
        self.root
            .borrow_mut(session)?
            .try_map(|value| self.selector.select_mut(value))
    }
}

impl<R, S> DynMapping for StaticMappingMut<R, S>
where
    R: StoreMut + Sync,
    S: SelectorMut<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn root(&self) -> &dyn DynStore {
        &self.root
    }
}

impl<R, S> DynMappingMut for StaticMappingMut<R, S>
where
    R: StoreMut + Sync,
    S: SelectorMut<R::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn root_mut(&self) -> &dyn DynStoreMut {
        &self.root
    }
}
