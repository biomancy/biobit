use eyre::{Result, ensure, eyre};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use substratum_compress::Decoder;

use super::indexed_reader::{IndexedRead, IndexedReader, IndexedReaderMutOp};

/// Reusable description of indexed FASTA input(s) that can open fresh readers on demand.
///
/// Construction only records paths and compression metadata. File and index validation happens in
/// [`IndexedSources::open`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedSources {
    sources: Vec<(PathBuf, Decoder)>,
}

impl IndexedSources {
    pub fn new(sources: Vec<(PathBuf, Decoder)>) -> Self {
        Self { sources }
    }

    pub fn from_path(fasta: impl AsRef<Path>, compression: Decoder) -> Self {
        Self {
            sources: vec![(fasta.as_ref().to_path_buf(), compression)],
        }
    }

    pub fn from_paths(indexed: &[(impl AsRef<Path>, Decoder)]) -> Self {
        let sources = indexed
            .iter()
            .map(|(fasta, compression)| (fasta.as_ref().to_path_buf(), *compression))
            .collect();
        Self { sources }
    }

    pub fn sources(&self) -> &[(PathBuf, Decoder)] {
        &self.sources
    }

    pub fn open(&self) -> Result<Box<dyn IndexedReaderMutOp + Send + Sync + 'static>> {
        let mut parsed: Vec<(Box<dyn IndexedRead>, _)> = Vec::with_capacity(self.sources.len());

        for (fasta, compression) in &self.sources {
            let mut path = fasta.clone();
            let file = File::open(&path)?;

            let fname = path
                .file_name()
                .and_then(|x| x.to_str())
                .unwrap_or_default()
                .to_string();
            path.set_file_name(format!("{fname}.fai"));
            ensure!(path.exists(), "fai index does not exist: {:?}", path);
            let fai = BufReader::new(File::open(&path)?);

            match compression {
                Decoder::Identity(_) => {
                    parsed.push((Box::new(file), fai));
                }
                Decoder::Bgzf(_) => {
                    path.set_file_name(format!("{fname}.gzi"));
                    ensure!(path.exists(), "gzi index does not exist: {:?}", path);
                    let gzi = noodles::bgzf::gzi::fs::read(&path)?;

                    let reader = Box::new(noodles::bgzf::io::indexed_reader::IndexedReader::new(
                        file, gzi,
                    ));
                    parsed.push((reader, fai));
                }
                _ => {
                    return Err(eyre!(
                        "Unsupported compression {:?} for an Indexed FASTA file: {}",
                        compression,
                        fasta.display()
                    ));
                }
            };
        }

        Ok(Box::new(IndexedReader::new(parsed)?))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use substratum_compress::Decoder;

    use super::*;

    #[test]
    fn construction_does_not_validate_paths() {
        let sources = IndexedSources::from_path("missing.fa", Decoder::default());

        assert_eq!(sources.sources().len(), 1);
        assert_eq!(sources.sources()[0].0.as_path(), Path::new("missing.fa"));
        assert!(sources.open().is_err());
    }
}
