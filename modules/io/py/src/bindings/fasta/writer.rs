pub use biobit_io_rs::fasta::{Writer, DEFAULT_LINE_WIDTH};
use std::num::NonZeroUsize;

use super::record::PyRecord;
use biobit_io_rs::compression::encode;
use biobit_io_rs::fasta::Record;
use biobit_io_rs::WriteRecord;
use derive_more::Into;
use eyre::{OptionExt, Result};
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(name = "Writer")]
#[derive(Into)]
pub struct PyWriter {
    path: PathBuf,
    rs: Option<Box<dyn WriteRecord<Record = Record> + Send + Sync + 'static>>,
}

#[pymethods]
impl PyWriter {
    #[new]
    #[pyo3(signature = (path, line_width = None))]
    pub fn new(path: PathBuf, line_width: Option<NonZeroUsize>) -> Result<Self> {
        let line_width = line_width.unwrap_or(DEFAULT_LINE_WIDTH);
        let rs = Some(Writer::from_path(
            path.clone(),
            &encode::Config::infer_from_path(&path),
            line_width,
        )?);
        Ok(Self { path, rs })
    }

    pub fn write_record<'a>(
        mut slf: PyRefMut<'a, Self>,
        record: &PyRecord,
    ) -> Result<PyRefMut<'a, Self>> {
        slf.rs
            .as_mut()
            .ok_or_eyre("Writing to a closed writer")?
            .write_record(&record.rs)?;
        Ok(slf)
    }

    pub fn write_records<'a>(
        mut slf: PyRefMut<'a, Self>,
        py: Python<'_>,
        records: Py<PyAny>,
    ) -> Result<PyRefMut<'a, Self>> {
        let rs = slf.rs.as_mut().ok_or_eyre("Writing to a closed writer")?;
        for record in records.bind(py).try_iter()? {
            let record = record?;
            let record = record.extract::<PyRecord>()?;
            rs.write_record(&record.rs)?;
        }
        Ok(slf)
    }

    pub fn flush(mut slf: PyRefMut<Self>) -> Result<PyRefMut<Self>> {
        slf.rs
            .as_mut()
            .ok_or_eyre("Writing to a closed writer")?
            .flush()?;
        Ok(slf)
    }

    pub fn close(mut slf: PyRefMut<Self>) -> Result<()> {
        let rs = slf.rs.take().ok_or_eyre("Writing to a closed writer")?;
        drop(rs);
        Ok(())
    }

    fn __enter__(slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf
    }

    fn __exit__(
        slf: PyRefMut<Self>,
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> Result<()> {
        Self::close(slf)?;
        Ok(())
    }
}
