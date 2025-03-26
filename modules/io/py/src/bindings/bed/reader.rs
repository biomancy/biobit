use super::record::{PyBed12, PyBed3, PyBed4, PyBed5, PyBed6, PyBed8, PyBed9};
use biobit_io_rs::bed::{Bed12, Bed3, Bed4, Bed5, Bed6, Bed8, Bed9, Reader};
use biobit_io_rs::compression::decode;
use biobit_io_rs::ReadRecord;
use derive_getters::Dissolve;
use derive_more::{From, Into};
use eyre::Result;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::path::PathBuf;

#[pyclass(name = "Reader")]
pub struct PyReader {}

#[pymethods]
impl PyReader {
    #[staticmethod]
    fn bed3(path: PathBuf) -> Result<PyBed3Reader> {
        let rs = Reader::from_path::<Bed3>(&path, &decode::Config::infer_from_path(&path))?;
        Ok(PyBed3Reader { path, rs })
    }

    #[staticmethod]
    fn bed4(path: PathBuf) -> Result<PyBed4Reader> {
        let rs = Reader::from_path::<Bed4>(&path, &decode::Config::infer_from_path(&path))?;
        Ok(PyBed4Reader { path, rs })
    }

    #[staticmethod]
    fn bed5(path: PathBuf) -> Result<PyBed5Reader> {
        let rs = Reader::from_path::<Bed5>(&path, &decode::Config::infer_from_path(&path))?;
        Ok(PyBed5Reader { path, rs })
    }

    #[staticmethod]
    fn bed6(path: PathBuf) -> Result<PyBed6Reader> {
        let rs = Reader::from_path::<Bed6>(&path, &decode::Config::infer_from_path(&path))?;
        Ok(PyBed6Reader { path, rs })
    }

    #[staticmethod]
    fn bed8(path: PathBuf) -> Result<PyBed8Reader> {
        let rs = Reader::from_path::<Bed8>(&path, &decode::Config::infer_from_path(&path))?;
        Ok(PyBed8Reader { path, rs })
    }

    #[staticmethod]
    fn bed9(path: PathBuf) -> Result<PyBed9Reader> {
        let rs = Reader::from_path::<Bed9>(&path, &decode::Config::infer_from_path(&path))?;
        Ok(PyBed9Reader { path, rs })
    }

    #[staticmethod]
    fn bed12(path: PathBuf) -> Result<PyBed12Reader> {
        let rs = Reader::from_path::<Bed12>(&path, &decode::Config::infer_from_path(&path))?;
        Ok(PyBed12Reader { path, rs })
    }
}

macro_rules! impl_bed_reader {
    ($Reader:ident, $Bed:ident, $PyBed:ident, $Name:literal) => {
        #[pyclass(name = $Name)]
        #[derive(Dissolve, From, Into)]
        pub struct $Reader {
            pub path: PathBuf,
            pub rs: Box<dyn ReadRecord<Record = $Bed> + Send + Sync + 'static>,
        }

        #[pymethods]
        impl $Reader {
            #[pyo3(signature = (into=None))]
            fn read_record(
                &mut self,
                py: Python,
                into: Option<Py<$PyBed>>,
            ) -> Result<Option<Py<$PyBed>>> {
                let into = match into {
                    Some(into) => into,
                    None => Py::new(py, $PyBed::from($Bed::default()))?,
                };

                let success = self.rs.read_record(&mut into.borrow_mut(py).rs)?;
                if success {
                    Ok(Some(into))
                } else {
                    Ok(None)
                }
            }

            fn read_to_end(&mut self) -> PyResult<Py<PyList>> {
                let mut result = Vec::new();
                self.rs.read_to_end(&mut result)?;

                let iterator = result.into_iter().map(|x| $PyBed::from(x));
                Python::with_gil(|py| -> PyResult<_> {
                    PyList::new(py, iterator).map(|x| x.unbind())
                })
            }

            fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
                slf
            }

            fn __next__(&mut self, py: Python) -> Result<Option<Py<$PyBed>>> {
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
