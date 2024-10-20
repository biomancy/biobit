use std::path::{Path, PathBuf};

use derive_getters::Dissolve;
use derive_more::{From, Into};
use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3::types::PyString;

use biobit_io_rs::bam::{Reader, ReaderBuilder};

#[derive(Debug, Into, From, Dissolve)]
pub struct IntoPyReader(pub Py<PyReader>);

impl<'py> FromPyObject<'py> for IntoPyReader {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        let reader = if obj.is_instance_of::<PyReader>() {
            obj.downcast::<PyReader>()?.clone().unbind()
        } else if obj.is_instance_of::<PyString>() {
            let filename = obj.extract::<PathBuf>()?;
            let reader: PyReader = ReaderBuilder::new(filename).build()?.into();

            Py::new(obj.py(), reader)?
        } else {
            return Err(PyValueError::new_err(format!("Unknown reader: {:?}", obj)));
        };

        Ok(reader.into())
    }
}

#[pyclass(eq, frozen, name = "Reader")]
#[repr(transparent)]
#[derive(Clone, PartialEq, From, Into, Dissolve)]
pub struct PyReader(pub Reader);

#[pymethods]
impl PyReader {
    #[new]
    #[pyo3(signature = (filename, inflags = 0, exflags = 516, minmapq = 0, batch_size = 1024))]
    pub fn new(
        filename: PathBuf,
        inflags: u16,
        exflags: u16,
        minmapq: u8,
        batch_size: usize,
    ) -> PyResult<Self> {
        if !filename.exists() {
            return Err(PyValueError::new_err(format!(
                "File not found: {:?}",
                filename
            )));
        }

        let reader = ReaderBuilder::new(filename)
            .with_inflags(inflags)
            .with_exflags(exflags)
            .with_minmapq(minmapq)
            .with_batch_size(batch_size)
            .build()?;
        Ok(Self(reader))
    }

    #[getter]
    pub fn filename(&self) -> &Path {
        self.0.filename()
    }

    #[getter]
    pub fn inflags(&self) -> u16 {
        *self.0.inflags()
    }

    #[getter]
    pub fn exflags(&self) -> u16 {
        *self.0.exflags()
    }

    #[getter]
    pub fn minmapq(&self) -> u8 {
        *self.0.minmapq()
    }

    #[getter]
    pub fn batch_size(&self) -> usize {
        *self.0.batch_size()
    }
}
