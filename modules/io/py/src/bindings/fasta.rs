use biobit_core_py::loc::{IntervalOp, IntoPyInterval};
use biobit_core_py::pickle;
use biobit_core_py::utils::ImportablePyModuleBuilder;
use biobit_io_rs::compression;
pub use biobit_io_rs::fasta::{
    IndexedReader, IndexedReaderMutOp, Reader, ReaderMutOp, Record, RecordMutOp,
};
use bitcode::{Decode, Encode};
use derive_getters::Dissolve;
use derive_more::{From, Into};
use eyre::{ContextCompat, Result};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};
use pyo3::PyTypeInfo;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyRecord>()?
        .add_class::<PyReader>()?
        .add_class::<PyIndexedReader>()?
        .finish();

    Ok(module)
}

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

#[pyclass(name = "Reader")]
#[derive(Dissolve, From, Into)]
pub struct PyReader {
    pub path: PathBuf,
    pub rs: Reader<Box<dyn std::io::BufRead + Send + Sync>>,
}

#[pymethods]
impl PyReader {
    #[new]
    fn new(path: PathBuf) -> Result<Self> {
        let bufread = compression::read_file(&path)?.box_bufread();
        let rs = Reader::new(bufread)?;

        Ok(Self { path, rs })
    }

    #[pyo3(signature = (into=None))]
    fn read_record(
        &mut self,
        py: Python,
        into: Option<Py<PyRecord>>,
    ) -> Result<Option<Py<PyRecord>>> {
        let into = match into {
            Some(into) => into,
            None => Py::new(py, PyRecord::from(Record::default()))?,
        };

        let result = self.rs.read_record(&mut into.borrow_mut(py).rs)?;
        match result {
            Some(()) => Ok(Some(into)),
            None => Ok(None),
        }
    }

    fn read_to_end(&mut self) -> PyResult<Py<PyList>> {
        let mut result = Vec::new();
        self.rs.read_to_end(&mut result)?;

        let iterator = result.into_iter().map(PyRecord::from);
        Python::with_gil(|py| -> PyResult<_> { PyList::new(py, iterator).map(|x| x.unbind()) })
    }

    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Result<Option<Py<PyRecord>>> {
        self.read_record(py, None)
    }
}

#[pyclass(name = "IndexedReader")]
#[derive(Dissolve, From, Into)]
pub struct PyIndexedReader {
    pub path: PathBuf,
    pub rs: Box<dyn IndexedReaderMutOp + Send + Sync + 'static>,
}

#[pymethods]
impl PyIndexedReader {
    #[new]
    fn new(path: PathBuf) -> Result<Self> {
        let rs = IndexedReader::<()>::from_path(&path)?;

        Ok(Self { path, rs })
    }

    #[getter]
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn fetch(&mut self, seqid: &str, interval: IntoPyInterval) -> Result<String> {
        let interval = Python::with_gil(|py| interval.0.borrow(py).rs.cast::<u64>())
            .wrap_err("Failed to cast interval to u64")?;

        let mut buffer = Vec::with_capacity(interval.len() as usize);
        self.rs.fetch(seqid, interval, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    fn fetch_full_seq(&mut self, seqid: &str) -> Result<String> {
        let mut buffer = Vec::new();
        self.rs.fetch_full_seq(seqid, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}
