//! Bioproj-specific record indexes and typed-ID lookup adapters.

use pyo3::prelude::*;

pub mod macros;
pub(crate) use macros::{define_family_record_index, define_homogeneous_record_index};

/// A supported lookup key, including strings that cannot identify a record.
pub enum LookupKey<K> {
    Typed(K),
    Untyped(biobit_bioproj_rs::UntypedId),
    Missing,
}

pub fn parse_key<K>(
    key: &Bound<'_, PyAny>,
    expected: &'static str,
    typed: impl FnOnce(&Bound<'_, PyAny>) -> PyResult<K>,
) -> PyResult<LookupKey<K>> {
    if key.is_instance_of::<pyo3::types::PyString>() {
        // Malformed strings cannot name a member; they have normal missing-key
        // behavior. Record/ID constructors still validate their input strictly.
        return Ok(key
            .extract::<String>()
            .ok()
            .and_then(|value| biobit_bioproj_rs::UntypedId::new(value).ok())
            .map_or(LookupKey::Missing, LookupKey::Untyped));
    }
    typed(key)
        .map(LookupKey::Typed)
        .map_err(|_| pyo3::exceptions::PyTypeError::new_err(expected))
}
