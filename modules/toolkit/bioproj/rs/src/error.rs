use crate::UntypedId;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

/// The result of a fallible bioproj operation.
pub type Result<T> = std::result::Result<T, Error>;

/// A value, graph, or storage failure in a biological project description.
///
/// Graph failures can be matched directly. Storage failures retain their
/// underlying I/O or JSON error through [`std::error::Error::source`].
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    EmptyValue,
    DuplicateValue,
    InvalidValue {
        field: &'static str,
        reason: String,
    },
    DuplicateKey {
        collection: &'static str,
        key: String,
    },
    MissingReference {
        owner: &'static str,
        id: UntypedId,
        field: &'static str,
        target: &'static str,
        target_id: UntypedId,
    },
    KindMismatch {
        id: UntypedId,
        expected: String,
        actual: String,
    },
    AcquisitionOverlap {
        design: UntypedId,
        acquisition: UntypedId,
        first_unit: UntypedId,
        second_unit: UntypedId,
    },
    DatasetSubset {
        acquisition: UntypedId,
        subset: UntypedId,
        superset: UntypedId,
    },
    AssetOwnership {
        asset: UntypedId,
        first_acquisition: UntypedId,
        second_acquisition: UntypedId,
    },
    UnrepresentedAcquisition {
        id: UntypedId,
    },
    UnusedAsset {
        id: UntypedId,
    },
    UnsupportedSchema {
        found: String,
        supported: String,
    },
    Io {
        operation: &'static str,
        path: Option<PathBuf>,
        source: std::io::Error,
    },
    Json {
        operation: &'static str,
        source: serde_json::Error,
    },
}

impl Error {
    pub(crate) fn invalid(field: &'static str, reason: impl Into<String>) -> Self {
        Self::InvalidValue {
            field,
            reason: reason.into(),
        }
    }

    pub(crate) fn io(operation: &'static str, path: Option<&Path>, source: std::io::Error) -> Self {
        Self::Io {
            operation,
            path: path.map(Path::to_path_buf),
            source,
        }
    }

    pub(crate) fn json(operation: &'static str, source: serde_json::Error) -> Self {
        if source.is_io() {
            // Serde's conversion recovers the original stream error. Keeping it
            // as Io also makes error classification independent of buffering.
            Self::io(operation, None, source.into())
        } else {
            Self::Json { operation, source }
        }
    }

    pub(crate) fn missing(
        owner: &'static str,
        id: &UntypedId,
        field: &'static str,
        target: &'static str,
        target_id: &UntypedId,
    ) -> Self {
        Self::MissingReference {
            owner,
            id: id.clone(),
            field,
            target,
            target_id: target_id.clone(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyValue => f.write_str("Value must not be empty"),
            Self::DuplicateValue => f.write_str("Set must not contain duplicate values"),
            Self::InvalidValue { field, reason } => write!(f, "{field}: {reason}"),
            Self::DuplicateKey { collection, key } => {
                write!(f, "{collection} contains duplicate key '{key}'")
            }
            Self::MissingReference {
                owner,
                id,
                field,
                target,
                target_id,
            } => {
                write!(
                    f,
                    "{owner} '{id}'.{field} references unknown {target} '{target_id}'"
                )
            }
            Self::KindMismatch {
                id,
                expected,
                actual,
            } => {
                write!(f, "ID '{id}' expects {expected}, but identifies {actual}")
            }
            Self::AcquisitionOverlap {
                design,
                acquisition,
                first_unit,
                second_unit,
            } => {
                write!(
                    f,
                    "Design '{design}' uses Acquisition '{acquisition}' in both Unit '{first_unit}' and Unit '{second_unit}'"
                )
            }
            Self::DatasetSubset {
                acquisition,
                subset,
                superset,
            } => {
                write!(
                    f,
                    "Dataset '{subset}' is a strict file subset of Dataset '{superset}' for Acquisition '{acquisition}'"
                )
            }
            Self::AssetOwnership {
                asset,
                first_acquisition,
                second_acquisition,
            } => {
                write!(
                    f,
                    "Asset '{asset}' belongs to different Acquisitions '{first_acquisition}' and '{second_acquisition}'"
                )
            }
            Self::UnrepresentedAcquisition { id } => write!(f, "Acquisition '{id}' has no Dataset"),
            Self::UnusedAsset { id } => write!(f, "Asset '{id}' does not belong to any Dataset"),
            Self::UnsupportedSchema { found, supported } => write!(
                f,
                "unsupported bioproj schema '{found}'; this loader supports '{supported}'"
            ),
            Self::Io {
                operation,
                path,
                source,
            } => {
                write!(f, "failed to {operation}")?;
                if let Some(path) = path {
                    write!(f, " '{}'", path.display())?;
                }
                write!(f, ": {source}")
            }
            Self::Json { operation, source } => write!(f, "failed to {operation}: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(error: std::convert::Infallible) -> Self {
        match error {}
    }
}
