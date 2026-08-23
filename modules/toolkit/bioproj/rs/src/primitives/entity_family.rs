/// Defines the structural types shared by a closed family of heterogeneous
/// entities.
///
/// One declaration supplies the variant name, concrete entity type, and
/// concrete identifier type. The macro derives the root entity enum, its kind,
/// and the owned and borrowed identifier unions from that registry. Domain
/// behavior, storage lookup, and reference validation deliberately remain
/// ordinary exhaustive Rust code so a new variant still requires an explicit
/// semantic decision at each relevant boundary.
macro_rules! define_entity_family {
    (
        $(#[$family_attr:meta])*
        $visibility:vis family $entity:ident {
            kind: $kind:ident,
            id: $id:ident,
            id_ref: $id_ref:ident,
            kind_doc: $kind_doc:literal,
            variants: {
                $(
                    $(#[$variant_attr:meta])*
                    $variant:ident($concrete:ty, $concrete_id:ty)
                ),+ $(,)?
            }
        }
    ) => {
        $(#[$family_attr])*
        #[derive(
            Clone,
            Debug,
            Eq,
            PartialEq,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::kinded::Kinded,
            ::derive_more::From,
        )]
        #[kinded(
            kind = $kind,
            skip_derive(From, FromStr),
            derive(Hash),
            attrs(doc = $kind_doc)
        )]
        #[serde(tag = "type", deny_unknown_fields)]
        $visibility enum $entity {
            $(
                $(#[$variant_attr])*
                $variant($concrete),
            )+
        }

        impl $entity {
            /// Returns this entity's concrete borrowed identifier.
            pub fn id(&self) -> $id_ref<'_> {
                match self {
                    $(Self::$variant(entity) => $id_ref::$variant(entity.id()),)+
                }
            }

            /// Returns auxiliary, non-structural metadata.
            pub fn meta(&self) -> &$crate::Meta {
                match self {
                    $(Self::$variant(entity) => entity.meta(),)+
                }
            }

            /// Returns the optional human-readable description.
            pub fn description(
                &self,
            ) -> ::core::option::Option<&$crate::NonEmpty<::std::string::String>> {
                match self {
                    $(Self::$variant(entity) => entity.description(),)+
                }
            }
        }

        #[doc = concat!("The owned identifier of any concrete [`", stringify!($entity), "`].")]
        ///
        /// This resolved union serializes as a bare [`crate::UntypedId`] but
        /// cannot deserialize alone because its wire value omits the variant.
        /// The owning domain restores that variant while resolving references;
        /// tagging it here would duplicate information that could disagree
        /// with the referenced entity.
        #[derive(
            Clone,
            Debug,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            ::derive_more::From,
        )]
        $visibility enum $id {
            $(
                #[doc = concat!("The identifier of [`", stringify!($entity), "::", stringify!($variant), "`].")]
                $variant($concrete_id),
            )+
        }

        impl $id {
            /// Borrows this identifier as its concrete tagged reference.
            pub fn as_ref(&self) -> $id_ref<'_> {
                match self {
                    $(Self::$variant(id) => $id_ref::$variant(id),)+
                }
            }

            /// Returns the shared workspace-local identifier.
            pub fn as_untyped(&self) -> &$crate::UntypedId {
                self.as_ref().as_untyped()
            }

            /// Returns whether this identifier has the same type and value as
            /// the entity's identifier.
            pub fn matches(&self, entity: &$entity) -> bool {
                self.as_ref().matches(entity)
            }
        }

        impl ::kinded::Kinded for $id {
            type Kind = $kind;

            fn kind(&self) -> Self::Kind {
                ::kinded::Kinded::kind(&self.as_ref())
            }
        }

        impl ::serde::Serialize for $id {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                ::serde::Serialize::serialize(self.as_untyped(), serializer)
            }
        }

        #[doc = concat!("A borrowed identifier of any concrete [`", stringify!($entity), "`].")]
        #[derive(Clone, Copy, Debug, Eq, PartialEq, ::derive_more::From)]
        $visibility enum $id_ref<'a> {
            $(
                #[doc = concat!("The identifier of [`", stringify!($entity), "::", stringify!($variant), "`].")]
                $variant(&'a $concrete_id),
            )+
        }

        impl<'a> $id_ref<'a> {
            /// Returns whether this identifier has the same type and value as
            /// the entity's identifier.
            pub fn matches(self, entity: &$entity) -> bool {
                self == entity.id()
            }

            /// Returns the shared workspace-local identifier.
            pub fn as_untyped(self) -> &'a $crate::UntypedId {
                match self {
                    $(Self::$variant(id) => id.as_untyped(),)+
                }
            }

            /// Clones this borrowed identifier into its owned union.
            pub fn to_owned(self) -> $id {
                match self {
                    $(Self::$variant(id) => $id::$variant(id.clone()),)+
                }
            }
        }

        impl ::kinded::Kinded for $id_ref<'_> {
            type Kind = $kind;

            fn kind(&self) -> Self::Kind {
                match self {
                    $(Self::$variant(_) => $kind::$variant,)+
                }
            }
        }

        $(
            impl ::kinded::Kinded for $concrete {
                type Kind = $kind;

                fn kind(&self) -> Self::Kind {
                    $kind::$variant
                }
            }

            impl $concrete {
                /// Returns this entity's concrete kind.
                pub const fn kind(&self) -> $kind {
                    $kind::$variant
                }
            }

            impl ::kinded::Kinded for $concrete_id {
                type Kind = $kind;

                fn kind(&self) -> Self::Kind {
                    $kind::$variant
                }
            }

            impl $concrete_id {
                /// Returns this identifier's concrete entity kind.
                pub const fn kind(&self) -> $kind {
                    $kind::$variant
                }
            }
        )+
    };
}

pub(crate) use define_entity_family;
