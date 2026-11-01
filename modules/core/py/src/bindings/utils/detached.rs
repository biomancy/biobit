/// Extract a native-only pyclass by cloning it while detached from Python.
///
/// Reserve this for I/O or large array copies. Routine records should use
/// PyO3's generated `from_py_object` implementation to avoid the transition.
///
/// The class must use `skip_from_py_object`. Its `Clone` implementation must not
/// call Python or clone Python-owned handles. The borrow guard remains in the
/// calling scope, preventing mutation while the native clone runs.
#[macro_export]
macro_rules! impl_detached_clone {
    ($class:ty) => {
        impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for $class {
            type Error = ::pyo3::pyclass::PyClassGuardError<'a, 'py>;

            fn extract(
                obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>,
            ) -> ::std::result::Result<Self, Self::Error> {
                let borrowed = obj.extract::<::pyo3::pyclass::PyClassGuard<'_, Self>>()?;
                let native = &*borrowed;
                Ok(obj.py().detach(|| native.clone()))
            }
        }
    };
}
