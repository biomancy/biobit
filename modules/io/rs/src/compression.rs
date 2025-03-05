use eyre::{ensure, Result};
use flate2::read::MultiGzDecoder;
use infer;
use std::fs::File;
use std::path::Path;

pub enum DecompressedStream {
    PlainText(File),
    Gzip(MultiGzDecoder<File>),
}

impl DecompressedStream {
    pub fn box_read(self) -> Box<dyn std::io::Read + Send + Sync + 'static> {
        match self {
            DecompressedStream::PlainText(file) => Box::new(file),
            DecompressedStream::Gzip(decoder) => Box::new(decoder),
        }
    }

    pub fn box_bufread(self) -> Box<dyn std::io::BufRead + Send + Sync + 'static> {
        match self {
            DecompressedStream::PlainText(file) => Box::new(std::io::BufReader::new(file)),
            DecompressedStream::Gzip(decoder) => Box::new(std::io::BufReader::new(decoder)),
        }
    }
}

pub fn read_file(path: impl AsRef<Path>) -> Result<DecompressedStream> {
    let path = path.as_ref();
    ensure!(path.exists(), "File {} does not exist", path.display());

    let ext = match infer::get_from_path(path)? {
        Some(extension) => extension,
        None => return Ok(DecompressedStream::PlainText(File::open(path)?)),
    };

    let compression = match (ext.extension(), ext.mime_type()) {
        ("gz", "application/gzip") => {
            let file = File::open(path)?;
            let decoder = MultiGzDecoder::new(file);
            DecompressedStream::Gzip(decoder)
        }
        // Always assume plain text if there is no clear match
        _ => {
            let file = File::open(path)?;
            DecompressedStream::PlainText(file)
        }
    };

    Ok(compression)
}
