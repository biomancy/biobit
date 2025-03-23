use super::algorithm::Algorithm;
use super::container::Container;
use bitcode::{Decode, Encode};
use derive_getters::Getters;
use derive_more::Into;
use eyre::{bail, ensure, Result};
use std::path::Path;

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Getters, Into)]
pub struct Compression {
    container: Container,
    algorithm: Algorithm,
}

impl Compression {
    pub const NONE: Compression = Compression {
        container: Container::None,
        algorithm: Algorithm::None,
    };

    pub fn new(container: Container, algorithm: Algorithm) -> Result<Self> {
        match container {
            Container::None => {
                // All algorithms can be used as a raw stream of compressed bytes
                Ok(Compression {
                    algorithm,
                    container,
                })
            }
            Container::Gzip | Container::Bgzf => {
                // Only DEFLATE can be used with GZIP or BGZF containers
                ensure!(
                    algorithm == Algorithm::Deflate,
                    "Only Deflate algorithm can be used with GZIP container"
                );
                Ok(Compression {
                    algorithm,
                    container,
                })
            }
        }
    }

    pub fn mime(&self) -> Option<&'static str> {
        match (self.container, self.algorithm) {
            (Container::Gzip, Algorithm::Deflate) => Some("application/gzip"),
            (Container::Bgzf, Algorithm::Deflate) => Some("application/gzip"),
            _ => None,
        }
    }

    pub fn infer_from_extension(path: impl AsRef<Path>) -> Result<Self> {
        match path.as_ref().extension() {
            Some(ext) => match ext.to_str() {
                Some("gz") | Some("gzip") => {
                    Ok(Compression::new(Container::Gzip, Algorithm::Deflate)?)
                }
                Some("bgz") | Some("bgzf") => {
                    Ok(Compression::new(Container::Bgzf, Algorithm::Deflate)?)
                }
                _ => Ok(Compression::new(Container::None, Algorithm::None)?),
            },
            None => Ok(Compression::new(Container::None, Algorithm::None)?),
        }
    }

    pub fn infer_from_file(file: impl AsRef<Path>) -> Result<Self> {
        let file = file.as_ref();
        ensure!(file.is_file(), "Path {} is not a file", file.display());

        let from_path = Compression::infer_from_extension(file)?;
        let mime = infer::get_from_path(file)?.map(|ext| ext.mime_type());

        // If the extension and MIME type match, return the inferred compression. Otherwise, throw an error
        if from_path.mime() == mime {
            Ok(from_path)
        } else {
            bail!(
                "Unknown compression format for file: {}.\n\
                Inferred mime values are {:?} (from path) and {:?} (infer crate)",
                file.display(),
                mime,
                from_path.mime()
            );
        }
    }
}
