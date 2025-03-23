use crate::compression::{Algorithm, Compression, Container};
use eyre::Result;
use noodles::bgzf;
use std::io::Write;

pub enum Encoder<W: Write + Send + Sync + 'static> {
    Raw(W),
    Deflate(flate2::write::DeflateEncoder<W>),
    Gzip(flate2::write::GzEncoder<W>),
    Bgzf(bgzf::Writer<W>),
}

impl<W: Write + Send + Sync + 'static> Encoder<W> {
    pub fn new(inner: W, compression: Compression) -> Result<Self> {
        match compression.container() {
            Container::None => {
                // All algorithms can be used as a raw stream of compressed bytes
                match compression.algorithm() {
                    Algorithm::None => Ok(Encoder::Raw(inner)),
                    Algorithm::Deflate => {
                        let encoder = flate2::write::DeflateEncoder::new(
                            inner,
                            flate2::Compression::default(),
                        );
                        Ok(Encoder::Deflate(encoder))
                    }
                }
            }
            Container::Gzip => {
                // Only DEFLATE can be used with GZIP or BGZF containers
                match compression.algorithm() {
                    Algorithm::Deflate => {
                        let encoder =
                            flate2::write::GzEncoder::new(inner, flate2::Compression::default());
                        Ok(Encoder::Gzip(encoder))
                    }
                    _ => Err(eyre::eyre!(
                        "Only Deflate algorithm can be used with GZIP container"
                    )),
                }
            }
            Container::Bgzf => {
                // Only DEFLATE can be used with GZIP or BGZF containers
                match compression.algorithm() {
                    Algorithm::Deflate => {
                        let writer = bgzf::Writer::new(inner);
                        Ok(Encoder::Bgzf(writer))
                    }
                    _ => Err(eyre::eyre!(
                        "Only Deflate algorithm can be used with BGZF container"
                    )),
                }
            }
        }
    }

    pub fn boxed(self) -> Box<dyn Write + Send + Sync + 'static> {
        match self {
            Encoder::Raw(file) => Box::new(file),
            Encoder::Gzip(encoder) => Box::new(encoder),
            Encoder::Bgzf(writer) => Box::new(writer),
            Encoder::Deflate(encoder) => Box::new(encoder),
        }
    }
}
