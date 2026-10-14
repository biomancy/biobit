/// Defines an entity enum, its ID and ID reference types from a list of variants.
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
        #[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
        #[derive(
            Clone,
            Debug,
            Eq,
            PartialEq,
            ::kinded::Kinded,
            ::derive_more::From,
        )]
        #[kinded(
            kind = $kind,
            skip_derive(From, FromStr),
            derive(Hash),
            attrs(
                doc = $kind_doc,
                cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))
            )
        )]
        $visibility enum $entity {
            $(
                $(#[$variant_attr])*
                $variant($concrete),
            )+
        }

        impl $entity {
            /// Returns this entity's ID.
            pub fn id(&self) -> $id_ref<'_> {
                match self {
                    $(Self::$variant(entity) => $id_ref::$variant(entity.id()),)+
                }
            }

            /// Returns this entity's metadata.
            pub fn meta(&self) -> &$crate::Meta {
                match self {
                    $(Self::$variant(entity) => entity.meta(),)+
                }
            }

            /// Returns the optional human-readable description.
            pub fn description(&self) -> ::core::option::Option<&$crate::Text> {
                match self {
                    $(Self::$variant(entity) => entity.description(),)+
                }
            }
        }

        #[doc = concat!("The owned identifier of any concrete [`", stringify!($entity), "`].")]
        #[cfg_attr(feature = "bitcode", derive(::bitcode::Encode, ::bitcode::Decode))]
        /// The ID includes the entity variant. Serde serializes an untyped ID;
        /// the owning collection restores the variant when loading references.
        /// Bitcode preserves the variant directly.
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
            /// Returns this ID as a borrowed ID type.
            pub fn as_ref(&self) -> $id_ref<'_> {
                match self {
                    $(Self::$variant(id) => $id_ref::$variant(id),)+
                }
            }

            /// Returns the untyped ID.
            pub fn as_untyped(&self) -> &$crate::UntypedId {
                self.as_ref().as_untyped()
            }
        }

        impl ::kinded::Kinded for $id {
            type Kind = $kind;

            fn kind(&self) -> Self::Kind {
                ::kinded::Kinded::kind(&self.as_ref())
            }
        }

        impl ::core::cmp::PartialEq<$id_ref<'_>> for $id {
            fn eq(&self, other: &$id_ref<'_>) -> bool {
                self.as_ref() == *other
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
        /// The borrowed ID includes the entity variant.
        #[derive(
            Clone,
            Copy,
            Debug,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            ::derive_more::From,
        )]
        $visibility enum $id_ref<'a> {
            $(
                #[doc = concat!("The identifier of [`", stringify!($entity), "::", stringify!($variant), "`].")]
                $variant(&'a $concrete_id),
            )+
        }

        impl<'a> $id_ref<'a> {
            /// Returns the untyped ID.
            pub fn as_untyped(self) -> &'a $crate::UntypedId {
                match self {
                    $(Self::$variant(id) => id.as_untyped(),)+
                }
            }

            /// Clones this borrowed ID into an owned ID.
            pub fn to_owned(&self) -> $id {
                match *self {
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

        impl ::core::cmp::PartialEq<$id> for $id_ref<'_> {
            fn eq(&self, other: &$id) -> bool {
                *self == other.as_ref()
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
                /// Returns this entity's kind.
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
                /// Returns this ID's entity kind.
                pub const fn kind(&self) -> $kind {
                    $kind::$variant
                }
            }
        )+
    };
}

pub(crate) use define_entity_family;

#[cfg(all(test, feature = "bitcode"))]
mod tests {
    use crate::data::{DatasetId, DatasetKind, dataset::fastq};

    #[test]
    fn bitcode_preserves_id_kinds() {
        for value in [DatasetKind::Fastq, DatasetKind::PairedFastq] {
            assert_eq!(
                bitcode::decode::<DatasetKind>(&bitcode::encode(&value)).unwrap(),
                value
            );
        }

        let single = fastq::SingleId::new("DATA1").unwrap();
        let paired = fastq::PairedId::new("DATA1").unwrap();
        assert_eq!(
            bitcode::decode::<fastq::SingleId>(&bitcode::encode(&single)).unwrap(),
            single
        );
        assert_eq!(
            bitcode::decode::<fastq::PairedId>(&bitcode::encode(&paired)).unwrap(),
            paired
        );

        let values = vec![DatasetId::Fastq(single), DatasetId::PairedFastq(paired)];
        assert_eq!(
            bitcode::decode::<Vec<DatasetId>>(&bitcode::encode(&values)).unwrap(),
            values
        );
    }
}
