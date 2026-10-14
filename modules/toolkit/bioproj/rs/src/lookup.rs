//! Typed lookups into project collections.

/// Result of looking up a typed ID.
///
/// `None` means the ID is missing. `Some(Ok(_))` contains a matching record;
/// `Some(Err(_))` means the ID belongs to another kind.
pub type OptResult<T, E> = Option<std::result::Result<T, E>>;

/// Looks up an entity by its typed ID.
///
/// Returns `None` if the ID is missing. A wrong-kind match returns an error.
/// Homogeneous collections use [`std::convert::Infallible`] as the error type.
///
/// Only project collections implement this trait.
pub trait Lookup<K: ?Sized>: sealed::Sealed<K> {
    /// The record type returned by the lookup.
    type Target: ?Sized;

    /// The error for a present ID of the wrong kind.
    type Err;

    /// Looks up `key`.
    fn lookup<'a>(&'a self, key: &K) -> OptResult<&'a Self::Target, Self::Err>;
}

/// Implements [`Lookup`] for a homogeneous map.
macro_rules! impl_direct_lookup {
    ($container:ty, $key:ty, $entity:ty, $field:ident) => {
        impl $crate::primitives::SealedLookup<$key> for $container {}

        impl $crate::Lookup<$key> for $container {
            type Target = $entity;
            type Err = ::std::convert::Infallible;

            fn lookup<'a>(
                &'a self,
                key: &$key,
            ) -> $crate::lookup::OptResult<&'a Self::Target, Self::Err> {
                self.$field.get(key).map(Ok)
            }
        }
    };
}

/// Implements [`Lookup`] for a heterogeneous map using the entity kind.
macro_rules! impl_checked_lookup {
    ($container:ty, $key:ty, $entity:ty, $field:ident) => {
        impl $crate::primitives::SealedLookup<$key> for $container {}

        impl $crate::Lookup<$key> for $container {
            type Target = $entity;
            type Err = $crate::Error;

            fn lookup<'a>(
                &'a self,
                key: &$key,
            ) -> $crate::lookup::OptResult<&'a Self::Target, Self::Err> {
                self.$field.get(key.as_untyped()).map(|entity| {
                    if ::kinded::Kinded::kind(key) == ::kinded::Kinded::kind(entity) {
                        Ok(entity)
                    } else {
                        Err($crate::Error::KindMismatch {
                            id: key.as_untyped().clone(),
                            expected: ::kinded::Kinded::kind(key).to_string(),
                            actual: ::kinded::Kinded::kind(entity).to_string(),
                        })
                    }
                })
            }
        }
    };
}

/// Implements [`Lookup`] for one variant in a heterogeneous map.
///
/// Returns an error when the ID belongs to another variant.
macro_rules! impl_variant_lookup {
    ($container:ty, $key:ty, $entity:ty, $field:ident, $family:ident::$variant:ident) => {
        impl $crate::primitives::SealedLookup<$key> for $container {}

        impl $crate::Lookup<$key> for $container {
            type Target = $entity;
            type Err = $crate::Error;

            fn lookup<'a>(
                &'a self,
                key: &$key,
            ) -> $crate::lookup::OptResult<&'a Self::Target, Self::Err> {
                self.$field.get(key.as_untyped()).map(|entity| {
                    #[allow(unreachable_patterns)]
                    match entity {
                        $family::$variant(entity) => Ok(entity),
                        entity => Err($crate::Error::KindMismatch {
                            id: key.as_untyped().clone(),
                            expected: ::kinded::Kinded::kind(key).to_string(),
                            actual: ::kinded::Kinded::kind(entity).to_string(),
                        }),
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
pub(crate) use impl_variant_lookup;
pub(crate) use sealed::Sealed;
