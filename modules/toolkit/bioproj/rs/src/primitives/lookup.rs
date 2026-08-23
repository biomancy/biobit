/// Looks up the entity identified by a particular ID type.
///
/// The container implements this trait because it owns both the storage and
/// the rules for navigating it. [`Lookup::Found`] is the value produced after
/// an ID was found: homogeneous collections return a reference directly,
/// while heterogeneous collections can return a [`Result`](eyre::Result) that
/// reports whether the stored enum variant agrees with the typed ID.
///
/// This trait is sealed because `bioproj` models a closed universe of entity
/// and ID types.
pub trait Lookup<K: ?Sized>: sealed::Sealed<K> {
    /// The value returned for a found ID.
    type Found<'a>
    where
        Self: 'a;

    /// Looks up `key`, returning `None` when its underlying ID is absent.
    fn lookup<'a>(&'a self, key: &K) -> Option<Self::Found<'a>>;
}

/// Implements [`Lookup`] for a homogeneous map whose key already determines
/// its stored entity type.
macro_rules! impl_direct_lookup {
    ($container:ty, $key:ty, $entity:ty, $field:ident) => {
        impl $crate::primitives::SealedLookup<$key> for $container {}

        impl $crate::Lookup<$key> for $container {
            type Found<'a> = &'a $entity;

            fn lookup<'a>(&'a self, key: &$key) -> Option<Self::Found<'a>> {
                self.$field.get(key)
            }
        }
    };
}

/// Implements [`Lookup`] for a heterogeneous map using the tagged ID's
/// strict `matches` contract and its `kind` values for mismatch diagnostics.
macro_rules! impl_checked_lookup {
    ($container:ty, $key:ty, $entity:ty, $field:ident) => {
        impl $crate::primitives::SealedLookup<$key> for $container {}

        impl $crate::Lookup<$key> for $container {
            type Found<'a> = ::eyre::Result<&'a $entity>;

            fn lookup<'a>(&'a self, key: &$key) -> Option<Self::Found<'a>> {
                self.$field.get(key.as_untyped()).map(|entity| {
                    if key.matches(entity) {
                        Ok(entity)
                    } else {
                        Err(::eyre::eyre!(
                            "ID '{}' expects {}, but identifies {}",
                            key.as_untyped(),
                            key.kind(),
                            entity.kind()
                        ))
                    }
                })
            }
        }
    };
}

mod sealed {
    pub trait Sealed<K: ?Sized> {}
}

pub(crate) use impl_checked_lookup;
pub(crate) use impl_direct_lookup;
pub(crate) use sealed::Sealed;
