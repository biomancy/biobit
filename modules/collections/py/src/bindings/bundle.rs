use std::collections::HashMap;
use std::hash::Hash;

use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[derive(Debug, Dissolve, Into, From)]
pub struct IntoPyBundle<K: Hash + Eq, V>(HashMap<K, V>);

impl<'py, K: Hash + Eq + FromPyObject<'py>, V: FromPyObject<'py>> FromPyObject<'py>
    for IntoPyBundle<K, V>
{
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let result = if obj.is_instance_of::<PyDict>() {
            obj.extract::<HashMap<K, V>>()?
        } else if obj.is_instance_of::<PyList>() {
            let list = obj.extract::<Vec<(K, V)>>()?;
            list.into_iter().collect()
        } else {
            return Err(PyValueError::new_err(format!(
                "Unknown bundle type: {}",
                obj
            )));
        };
        Ok(result.into())
    }
}
