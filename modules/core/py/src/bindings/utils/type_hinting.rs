use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use std::ffi::CString;

pub fn type_hint_class_getitem(cls: Bound<PyType>, args: PyObject) -> PyResult<PyObject> {
    let py = cls.py();
    let locals = PyDict::new(py);
    locals.set_item("cls", cls)?;
    locals.set_item("args", args)?;

    py.run(
        &CString::new(r#"import types;result = types.GenericAlias(cls, args);"#)?,
        None,
        Some(&locals),
    )?;
    let result = locals.get_item("result")?.ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Failed to get result for __class__getitem__",
        )
    })?;

    Ok(result.unbind())
}
