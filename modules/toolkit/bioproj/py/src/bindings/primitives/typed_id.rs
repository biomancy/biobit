macro_rules! define_id_binding {
    ($python:ident, $rust:path, $name:literal) => {
        #[pyclass(from_py_object, frozen, eq, ord, hash, name = $name)]
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $python(pub $crate::utils::frozen::Frozen<$rust>);

        $crate::utils::frozen::frozen_wrapper!($python, $rust);

        #[pymethods]
        impl $python {
            #[new]
            fn new(value: String) -> $crate::utils::error::PyBioprojResult<Self> {
                Ok(<$rust>::new(value)?.into())
            }

            fn __str__(&self) -> &str {
                self.0.as_ref().as_untyped().as_str()
            }

            fn __repr__(&self) -> String {
                format!(
                    concat!($name, "({:?})"),
                    self.0.as_ref().as_untyped().as_str()
                )
            }
        }
    };
}

pub(crate) use define_id_binding;
