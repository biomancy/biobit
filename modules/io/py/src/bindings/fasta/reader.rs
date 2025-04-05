use super::record::PyRecord;
use biobit_io_rs::compression::decode;
use biobit_io_rs::fasta::Record;
use biobit_io_rs::ReadRecord;
use derive_more::Into;
use eyre::Result;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::path::PathBuf;

pub use biobit_io_rs::fasta::Reader;

#[pyclass(name = "Reader")]
#[derive(Into)]
pub struct PyReader {
    path: PathBuf,
    rs: Box<dyn ReadRecord<Record = Record> + Send + Sync + 'static>,
}

#[pymethods]
impl PyReader {
    #[new]
    #[pyo3(signature = (path, compression=None))]
    fn new(path: PathBuf, compression: Option<&str>) -> Result<Self> {
        let config = match compression {
            None => decode::Config::infer_from_path(&path),
            Some(x) => decode::Config::infer_from_nickname(x)?,
        };
        let rs = Reader::from_path(path.clone(), &config)?;
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
        if self.rs.read_record(&mut into.borrow_mut(py).rs)? {
            Ok(Some(into))
        } else {
            Ok(None)
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
