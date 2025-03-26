use super::record::{PyBed12, PyBed3, PyBed4, PyBed5, PyBed6, PyBed8, PyBed9};
pub use biobit_io_rs::bed::Writer;
use biobit_io_rs::bed::{Bed12, Bed3, Bed4, Bed5, Bed6, Bed8, Bed9};
use biobit_io_rs::compression::encode;
use biobit_io_rs::WriteRecord;
use derive_more::Into;
use eyre::Result;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(name = "Writer")]
pub struct PyWriter {}

#[pymethods]
impl PyWriter {
    #[staticmethod]
    fn bed3(path: PathBuf) -> Result<PyBed3Writer> {
        let rs = Writer::from_path::<Bed3>(&path, &encode::Config::infer_from_path(&path))?;
        Ok(PyBed3Writer { path, rs })
    }

    #[staticmethod]
    fn bed4(path: PathBuf) -> Result<PyBed4Writer> {
        let rs = Writer::from_path::<Bed4>(&path, &encode::Config::infer_from_path(&path))?;
        Ok(PyBed4Writer { path, rs })
    }

    #[staticmethod]
    fn bed5(path: PathBuf) -> Result<PyBed5Writer> {
        let rs = Writer::from_path::<Bed5>(&path, &encode::Config::infer_from_path(&path))?;
        Ok(PyBed5Writer { path, rs })
    }

    #[staticmethod]
    fn bed6(path: PathBuf) -> Result<PyBed6Writer> {
        let rs = Writer::from_path::<Bed6>(&path, &encode::Config::infer_from_path(&path))?;
        Ok(PyBed6Writer { path, rs })
    }

    #[staticmethod]
    fn bed8(path: PathBuf) -> Result<PyBed8Writer> {
        let rs = Writer::from_path::<Bed8>(&path, &encode::Config::infer_from_path(&path))?;
        Ok(PyBed8Writer { path, rs })
    }

    #[staticmethod]
    fn bed9(path: PathBuf) -> Result<PyBed9Writer> {
        let rs = Writer::from_path::<Bed9>(&path, &encode::Config::infer_from_path(&path))?;
        Ok(PyBed9Writer { path, rs })
    }

    #[staticmethod]
    fn bed12(path: PathBuf) -> Result<PyBed12Writer> {
        let rs = Writer::from_path::<Bed12>(&path, &encode::Config::infer_from_path(&path))?;
        Ok(PyBed12Writer { path, rs })
    }
}

macro_rules! impl_bed_writer {
    ($Writer:ident, $Bed:ident, $PyBed:ident, $Name:literal) => {
        #[pyclass(name = $Name)]
        #[derive(Into)]
        pub struct $Writer {
            path: PathBuf,
            rs: Box<dyn WriteRecord<Record = $Bed> + Send + Sync + 'static>,
        }

        #[pymethods]
        impl $Writer {
            #[new]
            pub fn new(path: PathBuf) -> Result<Self> {
                let rs = Writer::from_path::<$Bed>(
                    path.clone(),
                    &encode::Config::infer_from_path(&path),
                )?;
                Ok(Self { path, rs })
            }

            pub fn write_record(&mut self, record: &$PyBed) -> Result<()> {
                self.rs.write_record(&record.rs)
            }

            pub fn write_records(&mut self, py: Python, records: Py<PyAny>) -> Result<()> {
                for record in records.bind(py).try_iter()? {
                    let record = record?;
                    let record = record.extract::<$PyBed>()?;
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
    };
}

impl_bed_writer!(PyBed3Writer, Bed3, PyBed3, "_Bed3Writer");
impl_bed_writer!(PyBed4Writer, Bed4, PyBed4, "_Bed4Writer");
impl_bed_writer!(PyBed5Writer, Bed5, PyBed5, "_Bed5Writer");
impl_bed_writer!(PyBed6Writer, Bed6, PyBed6, "_Bed6Writer");
impl_bed_writer!(PyBed8Writer, Bed8, PyBed8, "_Bed8Writer");
impl_bed_writer!(PyBed9Writer, Bed9, PyBed9, "_Bed9Writer");
impl_bed_writer!(PyBed12Writer, Bed12, PyBed12, "_Bed12Writer");
