use super::algorithm::Algorithm;
use super::config::Config;
use eyre::Result;
use noodles::bgzf;
use std::io::Write;

pub enum Stream<W: Write + Send + Sync + 'static> {
    Raw(W),
    Deflate(flate2::write::DeflateEncoder<W>),
    Gzip(flate2::write::GzEncoder<W>),
    Bgzf(bgzf::io::Writer<W>),
    MultithreadedBgzf(bgzf::io::MultithreadedWriter<W>),
}

impl<W: Write + Send + Sync + 'static> Stream<W> {
    pub fn new(inner: W, config: &Config) -> Result<Self> {
        match config {
            Config::RawBytes(algo) => match algo {
                Algorithm::None => Ok(Stream::Raw(inner)),
                Algorithm::Deflate(params) => {
                    let encoder = flate2::write::DeflateEncoder::new(
                        inner,
                        flate2::Compression::new(*params.level() as u32),
                    );
                    Ok(Stream::Deflate(encoder))
                }
            },
            Config::Gzip(params) => {
                let encoder = flate2::write::GzEncoder::new(
                    inner,
                    flate2::Compression::new(*params.level() as u32),
                );
                Ok(Stream::Gzip(encoder))
            }
            Config::Bgzf(params) => {
                let level =
                    bgzf::io::writer::CompressionLevel::new(*params.deflate().level()).unwrap();
                if params.threads().get() == 1 {
                    let writer = bgzf::io::writer::Builder::default()
                        .set_compression_level(level)
                        .build_from_writer(inner);
                    Ok(Stream::Bgzf(writer))
                } else {
                    let writer = bgzf::io::multithreaded_writer::Builder::default()
                        .set_compression_level(level)
                        .set_worker_count(*params.threads())
                        .build_from_writer(inner);
                    Ok(Stream::MultithreadedBgzf(writer))
                }
            }
        }
    }

    pub fn boxed(self) -> Box<dyn Write + Send + Sync + 'static> {
        match self {
            Stream::Raw(file) => Box::new(file),
            Stream::Gzip(encoder) => Box::new(encoder),
            Stream::Bgzf(writer) => Box::new(writer),
            Stream::MultithreadedBgzf(writer) => Box::new(writer),
            Stream::Deflate(encoder) => Box::new(encoder),
        }
    }
}
