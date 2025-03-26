use biobit_core_py::loc::{IntervalOp, IntoPyInterval};
use biobit_io_rs::compression::decode;
pub use biobit_io_rs::fasta::{IndexedReader, IndexedReaderMutOp};
use derive_more::Into;
use eyre::{ContextCompat, Result};
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(name = "IndexedReader")]
#[derive(Into)]
pub struct PyIndexedReader {
    pub path: PathBuf,
    pub rs: Box<dyn IndexedReaderMutOp + Send + Sync + 'static>,
}

#[pymethods]
impl PyIndexedReader {
    #[new]
    fn new(path: PathBuf) -> Result<Self> {
        let rs = IndexedReader::from_path(&path, &decode::Config::infer_from_path(&path))?;
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
