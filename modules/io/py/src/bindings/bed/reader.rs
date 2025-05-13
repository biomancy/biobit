use super::record::{PyBed3, PyBed4, PyBed5, PyBed6, PyBed8, PyBed9, PyBed12};
use biobit_io_rs::ReadRecord;
use biobit_io_rs::bed::{Bed3, Bed4, Bed5, Bed6, Bed8, Bed9, Bed12};
use biobit_io_rs::compression::decode;
use derive_more::Into;
use eyre::Result;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::path::PathBuf;

pub use biobit_io_rs::bed::Reader;

#[pyclass(name = "Reader")]
pub struct PyReader {}

#[pymethods]
impl PyReader {
    #[staticmethod]
    #[pyo3(signature = (path, compression=None))]
    fn bed3(path: PathBuf, compression: Option<&str>) -> Result<PyBed3Reader> {
        PyBed3Reader::new(path, compression)
    }

    #[staticmethod]
    #[pyo3(signature = (path, compression=None))]
    fn bed4(path: PathBuf, compression: Option<&str>) -> Result<PyBed4Reader> {
        PyBed4Reader::new(path, compression)
    }

    #[staticmethod]
    #[pyo3(signature = (path, compression=None))]
    fn bed5(path: PathBuf, compression: Option<&str>) -> Result<PyBed5Reader> {
        PyBed5Reader::new(path, compression)
    }

    #[staticmethod]
    #[pyo3(signature = (path, compression=None))]
    fn bed6(path: PathBuf, compression: Option<&str>) -> Result<PyBed6Reader> {
        PyBed6Reader::new(path, compression)
    }

    #[staticmethod]
    #[pyo3(signature = (path, compression=None))]
    fn bed8(path: PathBuf, compression: Option<&str>) -> Result<PyBed8Reader> {
        PyBed8Reader::new(path, compression)
    }

    #[staticmethod]
    #[pyo3(signature = (path, compression=None))]
    fn bed9(path: PathBuf, compression: Option<&str>) -> Result<PyBed9Reader> {
        PyBed9Reader::new(path, compression)
    }

    #[staticmethod]
    #[pyo3(signature = (path, compression=None))]
    fn bed12(path: PathBuf, compression: Option<&str>) -> Result<PyBed12Reader> {
        PyBed12Reader::new(path, compression)
    }
}

macro_rules! impl_bed_reader {
    ($Reader:ident, $Bed:ident, $PyBed:ident, $Name:literal) => {
        #[pyclass(name = $Name)]
        #[derive(Into)]
        pub struct $Reader {
            pub path: PathBuf,
            pub rs: Box<dyn ReadRecord<Record = $Bed> + Send + Sync + 'static>,
        }

        #[pymethods]
        impl $Reader {
            #[new]
            #[pyo3(signature = (path, compression=None))]
            pub fn new(path: PathBuf, compression: Option<&str>) -> Result<Self> {
                let config = match compression {
                    None => decode::Config::infer_from_path(&path),
                    Some(x) => decode::Config::infer_from_nickname(x)?,
                };
                let rs = Reader::from_path::<$Bed>(&path, &config)?;
                Ok(Self { path, rs })
            }

            #[pyo3(signature = (into=None))]
            pub fn read_record(
                &mut self,
                py: Python,
                into: Option<Py<$PyBed>>,
            ) -> Result<Option<Py<$PyBed>>> {
                let into = match into {
                    Some(into) => into,
                    None => Py::new(py, $PyBed::from($Bed::default()))?,
                };

                let success = self.rs.read_record(&mut into.borrow_mut(py).rs)?;
                if success { Ok(Some(into)) } else { Ok(None) }
            }

            pub fn read_to_end(&mut self) -> PyResult<Py<PyList>> {
                let mut result = Vec::new();
                self.rs.read_to_end(&mut result)?;

                let iterator = result.into_iter().map(|x| $PyBed::from(x));
                Python::with_gil(|py| -> PyResult<_> {
                    PyList::new(py, iterator).map(|x| x.unbind())
                })
            }

            pub fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
                slf
            }

            pub fn __next__(&mut self, py: Python) -> Result<Option<Py<$PyBed>>> {
                self.read_record(py, None)
            }
        }
    };
}

impl_bed_reader!(PyBed3Reader, Bed3, PyBed3, "_Bed3Reader");
impl_bed_reader!(PyBed4Reader, Bed4, PyBed4, "_Bed4Reader");
impl_bed_reader!(PyBed5Reader, Bed5, PyBed5, "_Bed5Reader");
impl_bed_reader!(PyBed6Reader, Bed6, PyBed6, "_Bed6Reader");
impl_bed_reader!(PyBed8Reader, Bed8, PyBed8, "_Bed8Reader");
impl_bed_reader!(PyBed9Reader, Bed9, PyBed9, "_Bed9Reader");
impl_bed_reader!(PyBed12Reader, Bed12, PyBed12, "_Bed12Reader");
