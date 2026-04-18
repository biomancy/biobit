use biobit_core_py::loc::{IntervalOp, IntoPyInterval};
pub use biobit_io_rs::fasta::{EXTENSIONS, IndexedReader, IndexedReaderMutOp};
use derive_more::Into;
use eyre::{ContextCompat, Result};
use pyo3::prelude::*;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use substratum_compress::Decoder;

#[pyclass(name = "IndexedReader")]
#[derive(Into)]
pub struct PyIndexedReader {
    pub paths: Vec<PathBuf>,
    pub rs: Box<dyn IndexedReaderMutOp + Send + Sync + 'static>,
}

#[pymethods]
impl PyIndexedReader {
    #[new]
    fn new(path: Bound<PyAny>) -> Result<Self> {
        // Could be either a single path or a list of paths
        let paths = if let Ok(path) = path.extract::<PathBuf>() {
            vec![path]
        } else {
            path.extract::<Vec<PathBuf>>()?
        };
        let decoders = paths
            .iter()
            .map(|x| Decoder::from_path(x, EXTENSIONS))
            .collect::<Result<Vec<_>, io::Error>>()?;
        let indexed: Vec<_> = paths.iter().zip(decoders).collect();
        let rs = IndexedReader::from_paths(&indexed)?;
        Ok(Self { paths, rs })
    }

    fn lengths(&self) -> HashMap<String, u64> {
        self.rs.lengths()
    }

    fn fetch(&mut self, seqid: &str, interval: IntoPyInterval) -> Result<String> {
        let interval = Python::attach(|py| interval.0.borrow(py).rs.cast::<u64>())
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
