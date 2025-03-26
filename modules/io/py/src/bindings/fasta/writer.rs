pub use biobit_io_rs::fasta::{Writer, DEFAULT_LINE_WIDTH};
use std::num::NonZeroUsize;

use super::record::PyRecord;
use biobit_io_rs::compression::encode;
use biobit_io_rs::fasta::Record;
use biobit_io_rs::WriteRecord;
use derive_more::Into;
use eyre::Result;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(name = "Writer")]
#[derive(Into)]
pub struct PyWriter {
    path: PathBuf,
    rs: Box<dyn WriteRecord<Record = Record> + Send + Sync + 'static>,
}

#[pymethods]
impl PyWriter {
    #[new]
    #[pyo3(signature = (path, line_width = None))]
    pub fn new(path: PathBuf, line_width: Option<NonZeroUsize>) -> Result<Self> {
        let line_width = line_width.unwrap_or(DEFAULT_LINE_WIDTH);
        let rs = Writer::from_path(
            path.clone(),
            &encode::Config::infer_from_path(&path),
            line_width,
        )?;
        Ok(Self { path, rs })
    }

    pub fn write_record(&mut self, record: &PyRecord) -> Result<()> {
        self.rs.write_record(&record.rs)
    }

    pub fn write_records(&mut self, py: Python, records: Py<PyAny>) -> Result<()> {
        for record in records.bind(py).try_iter()? {
            let record = record?;
            let record = record.extract::<PyRecord>()?;
            self.rs.write_record(&record.rs)?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.rs.flush()
    }

    fn __enter__(slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf
    }

    fn __exit__(
        mut slf: PyRefMut<Self>,
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> Result<()> {
        slf.flush()
    }
}
