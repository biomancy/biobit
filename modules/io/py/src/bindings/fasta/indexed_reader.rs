use biobit_core_py::loc::IntoPyInterval;
use biobit_core_py::utils::collections::Iterable;
pub use biobit_io_rs::fasta::{EXTENSIONS, IndexedReaderMutOp, IndexedSources};
use derive_more::Into;
use eyre::{ContextCompat, Result};
use pyo3::prelude::*;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use substratum_compress::Decoder;

#[pyclass(name = "IndexedSources", skip_from_py_object)]
#[derive(Clone, Into)]
pub struct PyIndexedSources {
    pub rs: IndexedSources,
}

#[pymethods]
impl PyIndexedSources {
    #[new]
    fn new(path: Bound<PyAny>) -> Result<Self> {
        let paths = if let Ok(path) = path.extract::<PathBuf>() {
            vec![path]
        } else {
            path.extract::<Iterable<PathBuf>>()?.into_iter().collect()
        };

        let sources = paths
            .into_iter()
            .map(|path| {
                let decoder = Decoder::from_path(&path, EXTENSIONS)?;
                Ok((path, decoder))
            })
            .collect::<Result<Vec<_>, io::Error>>()?;

        Ok(Self {
            rs: IndexedSources::new(sources),
        })
    }

    fn open(&self) -> Result<PyIndexedReader> {
        Ok(PyIndexedReader {
            rs: self.rs.open()?,
        })
    }
}

#[pyclass(name = "IndexedReader")]
#[derive(Into)]
pub struct PyIndexedReader {
    pub rs: Box<dyn IndexedReaderMutOp + Send + Sync + 'static>,
}

#[pymethods]
impl PyIndexedReader {
    fn lengths(&self) -> HashMap<String, u64> {
        self.rs.lengths()
    }

    fn fetch(&mut self, seqid: &str, interval: IntoPyInterval) -> Result<String> {
        let interval = Python::attach(|py| interval.0.borrow(py).rs.cast::<u64>())
            .wrap_err("Failed to cast interval to u64")?;

        let mut buffer = Vec::new();
        self.rs.fetch(seqid, interval, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    fn fetch_full_seq(&mut self, seqid: &str) -> Result<String> {
        let mut buffer = Vec::new();
        self.rs.fetch_full_seq(seqid, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}
