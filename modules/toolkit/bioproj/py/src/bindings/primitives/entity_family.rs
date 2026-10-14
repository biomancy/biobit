//! Python extraction and retained views for heterogeneous record families.

macro_rules! define_family_conversions {
    (
        $input:ident, $id_input:ident {
            rust: $rust:path,
            id: $id:path,
            id_ref: $id_ref:ident,
            variants: { $( $variant:ident => ($record:ident, $python_id:ident) ),+ $(,)? }
        }
    ) => {
        // Local adapters let PyO3 extract family inputs without implementing
        // foreign traits on the Rust domain's enum or exposing another pyclass.
        #[derive(pyo3::FromPyObject)]
        pub enum $input {
            $(#[pyo3(transparent)] $variant($record),)+
        }

        impl From<$input> for $rust {
            fn from(value: $input) -> Self {
                match value {
                    $($input::$variant(record) => Self::$variant(record.0.as_ref().clone()),)+
                }
            }
        }

        impl From<$crate::utils::frozen::Frozen<$rust>> for $input {
            fn from(value: $crate::utils::frozen::Frozen<$rust>) -> Self {
                use $rust as Family;
                match value.as_ref() {
                    $(Family::$variant(_) => {
                        let concrete = value.map(|family| {
                            #[allow(unreachable_patterns)]
                            match family {
                                Family::$variant(record) => record,
                                _ => unreachable!("a frozen family variant changed"),
                            }
                        });
                        Self::$variant($record::from(concrete))
                    },)+
                }
            }
        }

        impl<'py> pyo3::IntoPyObject<'py> for $input {
            type Target = pyo3::PyAny;
            type Output = pyo3::Bound<'py, pyo3::PyAny>;
            type Error = pyo3::PyErr;

            fn into_pyobject(self, py: pyo3::Python<'py>) -> pyo3::PyResult<Self::Output> {
                match self {
                    $(Self::$variant(record) => {
                        Ok(pyo3::Py::new(py, record)?.into_bound(py).into_any())
                    },)+
                }
            }
        }

        #[derive(pyo3::FromPyObject)]
        pub enum $id_input {
            $(#[pyo3(transparent)] $variant($python_id),)+
        }

        impl From<$id_input> for $id {
            fn from(value: $id_input) -> Self {
                match value {
                    $($id_input::$variant(id) => Self::$variant(id.0.as_ref().clone()),)+
                }
            }
        }

        /// Borrows the native ID for lookup without cloning its string.
        impl<'a> From<&'a $id_input> for $id_ref<'a> {
            fn from(value: &'a $id_input) -> Self {
                match value {
                    $($id_input::$variant(id) => Self::$variant(id.0.as_ref()),)+
                }
            }
        }

        impl $id_input {
            /// Selects a native ID and retains its owner in the concrete wrapper.
            /// The selector returns a temporary IdRef enum, so its concrete ID
            /// must be projected before Python object conversion can take place.
            pub fn from_owner<T: ?Sized + Send + Sync + 'static>(
                owner: &$crate::utils::frozen::Frozen<T>,
                selector: impl for<'a> Fn(&'a T) -> $id_ref<'a>,
            ) -> Self {
                match selector(owner.as_ref()) {
                    $($id_ref::$variant(_) => {
                        let id = owner.map(|value| {
                            #[allow(unreachable_patterns)]
                            match selector(value) {
                                $id_ref::$variant(id) => id,
                                _ => unreachable!("a frozen ID variant changed"),
                            }
                        });
                        Self::$variant($python_id::from(id))
                    },)+
                }
            }
        }

        impl<'py> pyo3::IntoPyObject<'py> for $id_input {
            type Target = pyo3::PyAny;
            type Output = pyo3::Bound<'py, pyo3::PyAny>;
            type Error = pyo3::PyErr;

            fn into_pyobject(self, py: pyo3::Python<'py>) -> pyo3::PyResult<Self::Output> {
                match self {
                    $(Self::$variant(id) => {
                        Ok(pyo3::Py::new(py, id)?.into_bound(py).into_any())
                    },)+
                }
            }
        }
    };
}

pub(crate) use define_family_conversions;
