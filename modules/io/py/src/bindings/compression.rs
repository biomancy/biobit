use std::path::Path;

use pyo3::PyResult;
use pyo3::exceptions::PyValueError;
use substratum_compress::Format;

/// Shared Python compression options. Inference uses only the filename suffix;
/// other suffixes mean uncompressed, while None always disables compression.
pub(crate) fn compression_format(path: &Path, compression: Option<&str>) -> PyResult<Format> {
    match compression {
        None => Ok(Format::Uncompressed),
        Some("gz") => Ok(Format::Gzip),
        Some("bgz") => Ok(Format::Bgzf),
        Some("infer") => Ok(match path.extension().and_then(|suffix| suffix.to_str()) {
            Some("gz" | "gzip") => Format::Gzip,
            Some("bgz" | "bgzf" | "bgzip") => Format::Bgzf,
            _ => Format::Uncompressed,
        }),
        Some(value) => Err(PyValueError::new_err(format!(
            "unsupported compression {value:?}; expected None, 'infer', 'gz', or 'bgz'"
        ))),
    }
}
