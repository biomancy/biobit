//! Pickle immutable wrappers by encoding their native value, not the Frozen owner.
//!
//! Restoration creates an independent owned value, including for wrappers that
//! originally selected a view into a larger project. Frozen wrappers cannot use
//! __setstate__; __reduce__ instead calls a constructor accepting bitcode bytes.

/// The value selector receives PyRef<Self>, giving subclasses access to their
/// base storage. Restoration defaults to Self::from(decoded_value). An optional
/// init expression can instead build a PyClassInitializer with both class layers.
macro_rules! pickle_methods {
    ($python:ident, $rust:ty, |$slf:ident| $value:expr $(,)?) => {
        $crate::utils::pickle::pickle_methods!(
            $python,
            $rust,
            |$slf| $value,
            init: |value| Self::from(value),
        );
    };
    (
        $python:ident, $rust:ty, |$slf:ident| $value:expr,
        init: |$decoded:ident| $initializer:expr $(,)?
    ) => {
        #[pyo3::pymethods]
        impl $python {
            #[staticmethod]
            fn _from_pickle(
                py: pyo3::Python<'_>,
                state: &pyo3::Bound<'_, pyo3::types::PyBytes>,
            ) -> pyo3::PyResult<pyo3::Py<Self>> {
                use pyo3::types::PyBytesMethods;
                let $decoded = biobit_core_py::pickle::from_bytes::<$rust>(state.as_bytes())
                    .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;
                pyo3::Py::new(py, $initializer)
            }

            fn __reduce__(
                $slf: pyo3::PyRef<'_, Self>,
                py: pyo3::Python<'_>,
            ) -> pyo3::PyResult<(pyo3::Py<pyo3::PyAny>, (Vec<u8>,))> {
                use pyo3::types::PyAnyMethods;
                let state = biobit_core_py::pickle::to_bytes::<$rust>(&$value);
                Ok((
                    py.get_type::<Self>().getattr("_from_pickle")?.unbind(),
                    (state,),
                ))
            }
        }
    };
}

pub(crate) use pickle_methods;
