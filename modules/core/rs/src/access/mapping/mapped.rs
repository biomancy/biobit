//! Inline selection over a mapping, including an erased parent.

use super::{ErasedMapping, ErasedMappingMut, Fallible, Infallible, Selector, SelectorMut, Then};
use crate::access::private;
use crate::access::{
    Access, AccessMut, DynAccess, DynAccessMut, DynMapping, DynMappingMut, DynStore, DynStoreMut,
    Kind, KindMut, Mapping, MappingMut, Read, Result, Session, Write,
};

/// An inline selector over a known mapping. Construction does not allocate.
/// Further maps compose selectors over the same parent; only erasure allocates.
#[derive(Clone)]
pub struct Mapped<M, S> {
    parent: M,
    selector: S,
}

impl<M, S> Mapped<M, S>
where
    M: Mapping + Sync + 'static,
    S: Selector<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    pub fn new(parent: M, selector: S) -> Self {
        Self { parent, selector }
    }

    pub fn root(&self) -> &dyn DynStore {
        self.parent.root()
    }

    /// Composes another selector without allocating or erasing its type.
    pub fn map<U: ?Sized + 'static, F>(self, next: F) -> Mapped<M, Then<S, Infallible<F>>>
    where
        F: for<'a> Fn(&'a S::Target) -> &'a U + Send + Sync + 'static,
    {
        Mapped::new(self.parent, Then::new(self.selector, Infallible::new(next)))
    }

    /// Composes a fallible selector without allocating.
    pub fn try_map<U: ?Sized + 'static, F>(self, next: F) -> Mapped<M, Then<S, Fallible<F>>>
    where
        F: for<'a> Fn(&'a S::Target) -> Result<&'a U> + Send + Sync + 'static,
    {
        Mapped::new(self.parent, Then::new(self.selector, Fallible::new(next)))
    }

    /// Shares the complete mapping in an Arc, erasing its concrete types.
    pub fn erase(self) -> ErasedMapping<S::Target> {
        ErasedMapping::new(self)
    }
}

impl<M, S> private::Sealed for Mapped<M, S> {}

impl<M, S> DynAccess for Mapped<M, S>
where
    M: Mapping + Sync + 'static,
    S: Selector<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn kind(&self) -> Kind<'_> {
        Kind::Mapping(self)
    }
}

impl<M, S> Access for Mapped<M, S>
where
    M: Mapping + Sync + 'static,
    S: Selector<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    type Target = S::Target;

    fn borrow<'a>(&'a self, session: &'a Session) -> Result<Read<'a, Self::Target>> {
        self.parent
            .borrow(session)?
            .try_map(|value| self.selector.select(value))
    }
}

impl<M, S> DynMapping for Mapped<M, S>
where
    M: Mapping + Sync + 'static,
    S: Selector<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn root(&self) -> &dyn DynStore {
        self.parent.root()
    }
}

/// An inline shared/mutable selector over a writable mapping.
/// It borrows under the parent's existing store-wide protection.
#[derive(Clone)]
pub struct MappedMut<M, S> {
    parent: M,
    selector: S,
}

impl<M, S> MappedMut<M, S>
where
    M: MappingMut + Sync + 'static,
    S: SelectorMut<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    pub fn new(parent: M, selector: S) -> Self {
        Self { parent, selector }
    }

    pub fn root(&self) -> &dyn DynStore {
        self.parent.root()
    }

    pub fn root_mut(&self) -> &dyn DynStoreMut {
        self.parent.root_mut()
    }

    /// Composes a shared selector without allocating.
    pub fn map<U: ?Sized + 'static, F>(self, next: F) -> Mapped<M, Then<S, Infallible<F>>>
    where
        F: for<'a> Fn(&'a S::Target) -> &'a U + Send + Sync + 'static,
    {
        self.into_read_only().map(next)
    }

    /// Composes a fallible shared selector without allocating.
    pub fn try_map<U: ?Sized + 'static, F>(self, next: F) -> Mapped<M, Then<S, Fallible<F>>>
    where
        F: for<'a> Fn(&'a S::Target) -> Result<&'a U> + Send + Sync + 'static,
    {
        self.into_read_only().try_map(next)
    }

    /// Composes shared and mutable selectors without allocating.
    pub fn map_mut<U: ?Sized + 'static, FR, FW>(
        self,
        read: FR,
        write: FW,
    ) -> MappedMut<M, Then<S, Infallible<FR, FW>>>
    where
        FR: for<'a> Fn(&'a S::Target) -> &'a U + Send + Sync + 'static,
        FW: for<'a> Fn(&'a mut S::Target) -> &'a mut U + Send + Sync + 'static,
    {
        MappedMut::new(
            self.parent,
            Then::new(self.selector, Infallible::new_mut(read, write)),
        )
    }

    /// Composes fallible shared and mutable selectors without allocating.
    pub fn try_map_mut<U: ?Sized + 'static, FR, FW>(
        self,
        read: FR,
        write: FW,
    ) -> MappedMut<M, Then<S, Fallible<FR, FW>>>
    where
        FR: for<'a> Fn(&'a S::Target) -> Result<&'a U> + Send + Sync + 'static,
        FW: for<'a> Fn(&'a mut S::Target) -> Result<&'a mut U> + Send + Sync + 'static,
    {
        MappedMut::new(
            self.parent,
            Then::new(self.selector, Fallible::new_mut(read, write)),
        )
    }

    /// Shares the complete writable mapping in an Arc, erasing its concrete types.
    pub fn erase(self) -> ErasedMappingMut<S::Target> {
        ErasedMappingMut::new(self)
    }

    /// Clones the parent and selector into a view with only read capability.
    pub fn read_only(&self) -> Mapped<M, S>
    where
        M: Clone,
        S: Clone,
    {
        Mapped::new(self.parent.clone(), self.selector.clone())
    }

    /// Transfers the parent and selector into a read-only view without cloning.
    pub fn into_read_only(self) -> Mapped<M, S> {
        Mapped::new(self.parent, self.selector)
    }
}

impl<M, S> private::Sealed for MappedMut<M, S> {}

impl<M, S> DynAccess for MappedMut<M, S>
where
    M: MappingMut + Sync + 'static,
    S: SelectorMut<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn kind(&self) -> Kind<'_> {
        Kind::Mapping(self)
    }
}

impl<M, S> DynAccessMut for MappedMut<M, S>
where
    M: MappingMut + Sync + 'static,
    S: SelectorMut<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn kind_mut(&self) -> KindMut<'_> {
        KindMut::Mapping(self)
    }
}

impl<M, S> Access for MappedMut<M, S>
where
    M: MappingMut + Sync + 'static,
    S: SelectorMut<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    type Target = S::Target;

    fn borrow<'a>(&'a self, session: &'a Session) -> Result<Read<'a, Self::Target>> {
        self.parent
            .borrow(session)?
            .try_map(|value| self.selector.select(value))
    }
}

impl<M, S> AccessMut for MappedMut<M, S>
where
    M: MappingMut + Sync + 'static,
    S: SelectorMut<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn borrow_mut<'a>(&'a self, session: &'a Session) -> Result<Write<'a, Self::Target>> {
        self.parent
            .borrow_mut(session)?
            .try_map(|value| self.selector.select_mut(value))
    }
}

impl<M, S> DynMapping for MappedMut<M, S>
where
    M: MappingMut + Sync + 'static,
    S: SelectorMut<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn root(&self) -> &dyn DynStore {
        self.parent.root()
    }
}

impl<M, S> DynMappingMut for MappedMut<M, S>
where
    M: MappingMut + Sync + 'static,
    S: SelectorMut<M::Target> + Send + Sync + 'static,
    S::Target: 'static,
{
    fn root_mut(&self) -> &dyn DynStoreMut {
        self.parent.root_mut()
    }
}
