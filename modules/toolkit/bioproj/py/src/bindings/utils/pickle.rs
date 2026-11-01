//! Pickle immutable wrappers by encoding their native value, not the Frozen owner.
//!
//! Restoration creates an independent owned value, including for wrappers that
//! originally selected a view into a larger project. Frozen wrappers cannot use
//! __setstate__; __reduce__ instead calls a constructor accepting bitcode bytes.

/// The value selector receives PyRef<Self>, giving subclasses access to their
/// base storage. Restoration defaults to Self::from(decoded_value). An optional
/// init expression can instead build a PyClassInitializer with both class layers.
///
/// Only aggregate project/index payloads opt into detachment; small records and
/// IDs use the inline codec to avoid a runtime transition for a few fields.
macro_rules! pickle_methods {
    ($python:ident, $rust:ty, |$slf:ident| $value:expr $(,)?) => {
        $crate::utils::pickle::pickle_methods!(
            $python,
            $rust,
            |$slf| $value,
            init: |value| Self::from(value),
            detached: false,
        );
    };
    ($python:ident, $rust:ty, |$slf:ident| $value:expr, detached: $detach:literal $(,)?) => {
        $crate::utils::pickle::pickle_methods!(
            $python, $rust, |$slf| $value,
            init: |value| Self::from(value), detached: $detach,
        );
    };
    (
        $python:ident, $rust:ty, |$slf:ident| $value:expr,
        init: |$decoded:ident| $initializer:expr $(,)?
    ) => {
        $crate::utils::pickle::pickle_methods!(
            $python, $rust, |$slf| $value,
            init: |$decoded| $initializer, detached: false,
        );
    };
    (
        $python:ident, $rust:ty, |$slf:ident| $value:expr,
        init: |$decoded:ident| $initializer:expr, detached: $detach:literal $(,)?
    ) => {
        #[pyo3::pymethods]
        impl $python {
            #[staticmethod]
            fn _from_pickle(
                py: pyo3::Python<'_>,
                state: &pyo3::Bound<'_, pyo3::types::PyBytes>,
            ) -> pyo3::PyResult<pyo3::Py<Self>> {
                use pyo3::types::PyBytesMethods;
                let bytes = state.as_bytes();
                let $decoded = if $detach {
                    py.detach(|| biobit_core_py::pickle::from_bytes::<$rust>(bytes))
                } else {
                    biobit_core_py::pickle::from_bytes::<$rust>(bytes)
                }
                .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;
                pyo3::Py::new(py, $initializer)
            }

            fn __reduce__(
                $slf: pyo3::PyRef<'_, Self>,
                py: pyo3::Python<'_>,
            ) -> pyo3::PyResult<(pyo3::Py<pyo3::PyAny>, (Vec<u8>,))> {
                use pyo3::types::PyAnyMethods;
                let value: &$rust = &$value;
                let state = if $detach {
                    py.detach(|| biobit_core_py::pickle::to_bytes::<$rust>(value))
                } else {
                    biobit_core_py::pickle::to_bytes::<$rust>(value)
                };
                Ok((
                    py.get_type::<Self>().getattr("_from_pickle")?.unbind(),
                    (state,),
                ))
            }
        }
    };
}

pub(crate) use pickle_methods;
