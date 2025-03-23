use super::algorithm::Algorithm;
use super::compression::Compression;
use super::container::Container;
use eyre::{ensure, Result};
use flate2;
use std::io::Read;

// Currently, the decoder is very simple and only supports the Read interface.
// Technically, we can create BufDecoder, SeekDecoder, BufSeekDecoder, but...
// * It is unclear if/how to handle indexed compressed files except for BGZF.
// * It is unclear how often compression libraries offer BufRead interfaces. Maybe end users should simply
//   wrap the decoder in a BufReader if they need it.

pub enum Decoder<R: Read + Send + Sync + 'static> {
    Raw(R),
    Gz(flate2::read::MultiGzDecoder<R>), // gzip and bgzf
    Deflate(flate2::read::DeflateDecoder<R>),
}

impl<R: Read + Send + Sync + 'static> Decoder<R> {
    pub fn new(inner: R, compression: Compression) -> Result<Self> {
        match compression.container() {
            // All algorithms can be used as a raw stream of compressed bytes
            Container::None => match compression.algorithm() {
                Algorithm::None => Ok(Decoder::Raw(inner)),
                Algorithm::Deflate => {
                    let decoder = flate2::read::DeflateDecoder::new(inner);
                    Ok(Decoder::Deflate(decoder))
                }
            },
            // Only DEFLATE can be used with GZIP or BGZF containers
            Container::Gzip | Container::Bgzf => {
                ensure!(
                    *compression.algorithm() == Algorithm::Deflate,
                    "Only DEFLATE algorithm can be used with GZIP and BGZF containers"
                );
                let decoder = flate2::read::MultiGzDecoder::new(inner);
                Ok(Decoder::Gz(decoder))
            }
        }
    }

    pub fn boxed(self) -> Box<dyn Read + Send + Sync + 'static> {
        match self {
            Decoder::Raw(stream) => Box::new(stream),
            Decoder::Gz(stream) => Box::new(stream),
            Decoder::Deflate(stream) => Box::new(stream),
        }
    }
}
