use std::sync::Arc;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use biobit_reat_rs::selection::Selector;

use super::{PyMismatches, PyRequiredOrMismatches, PyRequiredSites};

pub struct IntoPySelector {
    pub rs: Arc<dyn Selector<String, u64, u32> + Send + Sync>,
}

impl From<PyMismatches> for IntoPySelector {
    fn from(value: PyMismatches) -> Self {
        Self {
            rs: Arc::new(value.rs),
        }
    }
}

impl From<PyRequiredSites> for IntoPySelector {
    fn from(value: PyRequiredSites) -> Self {
        Self {
            rs: Arc::new(value.rs),
        }
    }
}

impl From<PyRequiredOrMismatches> for IntoPySelector {
    fn from(value: PyRequiredOrMismatches) -> Self {
        Self {
            rs: Arc::new(value.rs),
        }
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for IntoPySelector {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if obj.is_instance_of::<PyMismatches>() {
            let selector = *obj.cast::<PyMismatches>()?.borrow();
            Ok(selector.into())
        } else if obj.is_instance_of::<PyRequiredSites>() {
            let selector = obj.cast::<PyRequiredSites>()?.borrow().clone();
            Ok(selector.into())
        } else if obj.is_instance_of::<PyRequiredOrMismatches>() {
            let selector = obj.cast::<PyRequiredOrMismatches>()?.borrow().clone();
            Ok(selector.into())
        } else {
            Err(PyValueError::new_err(format!(
                "Unknown REAT selector: {}",
                obj.str()?
            )))
        }
    }
}
