use super::record::{PyBed12, PyBed3, PyBed4, PyBed5, PyBed6, PyBed8, PyBed9};
pub use biobit_io_rs::bed::Writer;
use biobit_io_rs::bed::{Bed12, Bed3, Bed4, Bed5, Bed6, Bed8, Bed9};
use biobit_io_rs::compression::encode;
use biobit_io_rs::WriteRecord;
use derive_more::Into;
use eyre::OptionExt;
use eyre::Result;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(name = "Writer")]
pub struct PyWriter {}

#[pymethods]
impl PyWriter {
    #[staticmethod]
    fn bed3(path: PathBuf) -> Result<PyBed3Writer> {
        let rs = Some(Writer::from_path::<Bed3>(
            &path,
            &encode::Config::infer_from_path(&path),
        )?);
        Ok(PyBed3Writer { path, rs })
    }

    #[staticmethod]
    fn bed4(path: PathBuf) -> Result<PyBed4Writer> {
        let rs = Some(Writer::from_path::<Bed4>(
            &path,
            &encode::Config::infer_from_path(&path),
        )?);
        Ok(PyBed4Writer { path, rs })
    }

    #[staticmethod]
    fn bed5(path: PathBuf) -> Result<PyBed5Writer> {
        let rs = Some(Writer::from_path::<Bed5>(
            &path,
            &encode::Config::infer_from_path(&path),
        )?);
        Ok(PyBed5Writer { path, rs })
    }

    #[staticmethod]
    fn bed6(path: PathBuf) -> Result<PyBed6Writer> {
        let rs = Some(Writer::from_path::<Bed6>(
            &path,
            &encode::Config::infer_from_path(&path),
        )?);
        Ok(PyBed6Writer { path, rs })
    }

    #[staticmethod]
    fn bed8(path: PathBuf) -> Result<PyBed8Writer> {
        let rs = Some(Writer::from_path::<Bed8>(
            &path,
            &encode::Config::infer_from_path(&path),
        )?);
        Ok(PyBed8Writer { path, rs })
    }

    #[staticmethod]
    fn bed9(path: PathBuf) -> Result<PyBed9Writer> {
        let rs = Some(Writer::from_path::<Bed9>(
            &path,
            &encode::Config::infer_from_path(&path),
        )?);
        Ok(PyBed9Writer { path, rs })
    }

    #[staticmethod]
    fn bed12(path: PathBuf) -> Result<PyBed12Writer> {
        let rs = Some(Writer::from_path::<Bed12>(
            &path,
            &encode::Config::infer_from_path(&path),
        )?);
        Ok(PyBed12Writer { path, rs })
    }
}

macro_rules! impl_bed_writer {
    ($Writer:ident, $Bed:ident, $PyBed:ident, $Name:literal) => {
        #[pyclass(name = $Name)]
        #[derive(Into)]
        pub struct $Writer {
            path: PathBuf,
            rs: Option<Box<dyn WriteRecord<Record = $Bed> + Send + Sync + 'static>>,
        }

        #[pymethods]
        impl $Writer {
            #[new]
            pub fn new(path: PathBuf) -> Result<Self> {
                let rs = Some(Writer::from_path::<$Bed>(
                    path.clone(),
                    &encode::Config::infer_from_path(&path),
                )?);
                Ok(Self { path, rs })
            }

            pub fn write_record<'a>(
                mut slf: PyRefMut<'a, Self>,
                record: &$PyBed,
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
                    let record = record.extract::<$PyBed>()?;
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
    };
}

impl_bed_writer!(PyBed3Writer, Bed3, PyBed3, "_Bed3Writer");
impl_bed_writer!(PyBed4Writer, Bed4, PyBed4, "_Bed4Writer");
impl_bed_writer!(PyBed5Writer, Bed5, PyBed5, "_Bed5Writer");
impl_bed_writer!(PyBed6Writer, Bed6, PyBed6, "_Bed6Writer");
impl_bed_writer!(PyBed8Writer, Bed8, PyBed8, "_Bed8Writer");
impl_bed_writer!(PyBed9Writer, Bed9, PyBed9, "_Bed9Writer");
impl_bed_writer!(PyBed12Writer, Bed12, PyBed12, "_Bed12Writer");
