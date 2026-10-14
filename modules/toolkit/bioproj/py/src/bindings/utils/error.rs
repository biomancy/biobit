use biobit_bioproj_rs::Error;
use derive_more::{From, Into};
use pyo3::PyErr;
use pyo3::exceptions::{PyIOError, PyKeyError, PyRuntimeError, PyTypeError, PyValueError};

#[derive(Debug, From, Into)]
pub struct PyBioprojErr(PyErr);

pub type PyBioprojResult<T> = Result<T, PyBioprojErr>;

impl From<Error> for PyBioprojErr {
    fn from(error: Error) -> Self {
        let message = error.to_string();
        Self(match error {
            Error::MissingReference { .. } => PyKeyError::new_err(message),
            Error::KindMismatch { .. } => PyTypeError::new_err(message),
            Error::Io { .. } => PyIOError::new_err(message),
            Error::EmptyValue
            | Error::DuplicateValue
            | Error::InvalidValue { .. }
            | Error::DuplicateKey { .. }
            | Error::AcquisitionOverlap { .. }
            | Error::DatasetSubset { .. }
            | Error::AssetOwnership { .. }
            | Error::UnrepresentedAcquisition { .. }
            | Error::UnusedAsset { .. }
            | Error::UnsupportedSchema { .. }
            | Error::Json { .. } => PyValueError::new_err(message),
            // Error is non-exhaustive; future variants still produce a useful message.
            _ => PyRuntimeError::new_err(message),
        })
    }
}
