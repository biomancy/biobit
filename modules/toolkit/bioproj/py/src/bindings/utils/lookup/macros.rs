// Both homogeneous and heterogeneous indexes share the mapping behavior.
// Callers supply only typed lookup and conversion into Python views.
macro_rules! define_record_index {
    (
        $index:ident($domain:ty) as $name:literal,
        rust: $rust:ty,
        lookup: $lookup:ident,
        iter: $iter:ident,
        get: |$get_domain:ident, $get_key:ident| $get:block,
        key: |$key_py:ident, $key_view:ident| $key:block,
        value: |$value_py:ident, $value_view:ident| $value:block
    ) => {
        #[pyclass(skip_from_py_object, frozen, name = $name)]
        #[derive(Clone)]
        pub struct $index(pub $crate::utils::frozen::Frozen<$domain>);

        impl $index {
            fn lookup_rust<'a>(
                $get_domain: &'a $domain,
                $get_key: &Bound<'_, PyAny>,
            ) -> $crate::utils::error::PyBioprojResult<Option<&'a $rust>> {
                $get
            }

            fn key_to_python(
                $key_py: Python<'_>,
                $key_view: $crate::utils::frozen::Frozen<$rust>,
            ) -> PyResult<Py<PyAny>> {
                $key
            }

            fn value_to_python(
                $value_py: Python<'_>,
                $value_view: $crate::utils::frozen::Frozen<$rust>,
            ) -> PyResult<Py<PyAny>> {
                $value
            }
        }

        impl biobit_core_py::utils::collections::PyMappingBackend for $index {
            fn len(&self) -> usize {
                self.0.as_ref().$iter().len()
            }

            fn iter_keys(&self) -> biobit_core_py::utils::collections::PyIterator {
                let owner = self.0.clone();
                biobit_core_py::utils::collections::py_iterator!(
                    self.0.clone(),
                    |source| source.as_ref().$iter(),
                    move |py, record| {
                        let id = record.id().as_untyped();
                        let view = owner.map(|domain| {
                            domain.$lookup(id).expect("an indexed record disappeared")
                        });
                        Self::key_to_python(py, view)
                    },
                )
            }

            fn lookup(
                &self,
                py: Python<'_>,
                key: &Bound<'_, PyAny>,
            ) -> PyResult<Option<Py<PyAny>>> {
                // None distinguishes a missing key from a Python error.
                // Retain the result of the lookup itself instead of looking it up again.
                let selected = self.0.try_map(|domain| {
                    Self::lookup_rust(domain, key)
                        .map_err(Some)
                        .and_then(|value| value.ok_or(None))
                });
                match selected {
                    Ok(view) => Self::value_to_python(py, view).map(Some),
                    Err(None) => Ok(None),
                    Err(Some(error)) => Err(error.into()),
                }
            }

            fn contains(&self, key: &Bound<'_, PyAny>) -> bool {
                Self::lookup_rust(self.0.as_ref(), key).is_ok_and(|value| value.is_some())
            }
        }

        biobit_core_py::utils::collections::py_mapping_methods!($index);
    };
}

macro_rules! define_homogeneous_record_index {
    (
        $index:ident($domain:ty) as $name:literal,
        record: $record:ident,
        id: $id:ident as $key_name:literal,
        rust: $rust:ty,
        lookup: $lookup:ident,
        iter: $iter:ident
    ) => {
        $crate::utils::lookup::macros::define_record_index! {
            $index($domain) as $name,
            rust: $rust,
            lookup: $lookup,
            iter: $iter,
            get: |domain, key| {
                use $crate::utils::lookup::LookupKey;
                Ok(match $crate::utils::lookup::parse_key(
                    key, concat!("expected ", $key_name, " or str"),
                    |key| Ok(key.extract::<$id>()?),
                )? {
                    LookupKey::Typed(id) => domain.$lookup(id.0.as_ref().as_untyped()),
                    LookupKey::Untyped(id) => domain.$lookup(&id),
                    LookupKey::Missing => None,
                })
            },
            key: |py, view| {
                Ok(Py::new(py, $id::from(view.map(<$rust>::id)))?.into_any())
            },
            value: |py, view| {
                Ok(Py::new(py, $record::from(view))?.into_any())
            }
        }
    };
}

macro_rules! define_family_record_index {
    (
        $index:ident($domain:ty) as $name:literal,
        record: $record:ident,
        id: $id:ident as $key_name:literal,
        id_ref: $id_ref:ty,
        rust: $rust:ty,
        lookup: $lookup:ident,
        iter: $iter:ident
    ) => {
        $crate::utils::lookup::macros::define_record_index! {
            $index($domain) as $name,
            rust: $rust,
            lookup: $lookup,
            iter: $iter,
            get: |domain, key| {
                use $crate::utils::lookup::LookupKey;
                Ok(match $crate::utils::lookup::parse_key(
                    key, concat!("expected ", $key_name, " or str"),
                    |key| key.extract::<$id>(),
                )? {
                    LookupKey::Typed(id) => {
                        let id: $id_ref = (&id).into();
                        domain.get(&id).transpose()?
                    },
                    LookupKey::Untyped(id) => domain.$lookup(&id),
                    LookupKey::Missing => None,
                })
            },
            key: |py, view| {
                pyo3::IntoPyObjectExt::into_py_any($id::from_owner(&view, <$rust>::id), py)
            },
            value: |py, view| {
                pyo3::IntoPyObjectExt::into_py_any($record::from(view), py)
            }
        }
    };
}

pub(crate) use define_family_record_index;
pub(crate) use define_homogeneous_record_index;
pub(crate) use define_record_index;
