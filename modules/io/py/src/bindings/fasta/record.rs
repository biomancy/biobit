use biobit_core_py::pickle;
pub use biobit_io_rs::fasta::{Record, RecordMutOp, RecordOp};
use bitcode::{Decode, Encode};
use derive_more::{From, Into};
use eyre::Result;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::PyTypeInfo;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

#[pyclass(eq, ord, name = "Record")]
#[repr(transparent)]
#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Into)]
pub struct PyRecord {
    pub rs: Record,
}

#[pymethods]
impl PyRecord {
    #[new]
    fn new(header: String, sequence: String) -> Result<Self> {
        Ok(Self {
            rs: Record::new(header, sequence.into_bytes())?,
        })
    }

    #[getter]
    fn id(&self) -> &str {
        self.rs.id()
    }

    #[setter]
    fn set_id(&mut self, id: String) -> Result<()> {
        self.rs.set_id(id)?;
        Ok(())
    }

    #[getter]
    fn seq(&self) -> String {
        String::from_utf8_lossy(self.rs.seq()).to_string()
    }

    #[setter]
    fn set_seq(&mut self, seq: String) -> Result<()> {
        self.rs.set_seq(seq.into_bytes())?;
        Ok(())
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> String {
        format!("Record({:?}, {:?})", self.rs.id(), self.rs.seq())
    }

    #[staticmethod]
    fn _from_pickle(state: &Bound<PyBytes>) -> PyResult<Self> {
        pickle::from_bytes(state.as_bytes()).map_err(|e| e.into())
    }

    fn __reduce__(&self, py: Python) -> Result<(PyObject, (Vec<u8>,))> {
        Ok((
            Self::type_object(py).getattr("_from_pickle")?.unbind(),
            (pickle::to_bytes(self),),
        ))
    }
}
