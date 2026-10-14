pub(crate) use biobit_core_rs::access::Frozen;

macro_rules! frozen_wrapper {
    ($python:ident, $rust:ty) => {
        impl From<$rust> for $python {
            fn from(value: $rust) -> Self {
                Self(value.into())
            }
        }

        impl From<$crate::utils::frozen::Frozen<$rust>> for $python {
            fn from(value: $crate::utils::frozen::Frozen<$rust>) -> Self {
                Self(value)
            }
        }

        impl std::borrow::Borrow<$rust> for $python {
            fn borrow(&self) -> &$rust {
                self.0.as_ref()
            }
        }
    };
}

pub(crate) use frozen_wrapper;
