/// Implements one static [`kinded::Kinded`] value for one or more concrete
/// types and provides each type with an inherent `kind` method.
macro_rules! impl_kind {
    ($kind:ty, $variant:ident => $($target:ty),+ $(,)?) => {
        $(
            impl ::kinded::Kinded for $target {
                type Kind = $kind;

                fn kind(&self) -> Self::Kind {
                    <$kind>::$variant
                }
            }

            impl $target {
                /// Returns this value's concrete kind.
                pub const fn kind(&self) -> $kind {
                    <$kind>::$variant
                }
            }
        )+
    };
}

pub(crate) use impl_kind;
