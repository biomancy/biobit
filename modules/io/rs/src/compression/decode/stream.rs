use super::algorithm::Algorithm;
use super::config::Config;
use eyre::Result;
use noodles::bgzf;
use std::io::Read;

pub enum Stream<R: Read + Send + Sync + 'static> {
    Raw(R),
    Deflate(flate2::read::DeflateDecoder<R>),
    Gzip(flate2::read::MultiGzDecoder<R>),
    Bgzf(bgzf::io::Reader<R>),
    MultithreadedBgzf(bgzf::io::MultithreadedReader<R>),
}

impl<R: Read + Send + Sync + 'static> Stream<R> {
    pub fn new(inner: R, config: &Config) -> Result<Self> {
        match config {
            Config::RawBytes(algo) => match algo {
                Algorithm::None => Ok(Stream::Raw(inner)),
                Algorithm::Deflate => Ok(Stream::Deflate(flate2::read::DeflateDecoder::new(inner))),
            },
            Config::Gzip => Ok(Stream::Gzip(flate2::read::MultiGzDecoder::new(inner))),
            Config::Bgzf(params) => {
                if params.threads().get() == 1 {
                    Ok(Stream::Bgzf(bgzf::io::Reader::new(inner)))
                } else {
                    Ok(Stream::MultithreadedBgzf(
                        bgzf::io::MultithreadedReader::with_worker_count(*params.threads(), inner),
                    ))
                }
            }
        }
    }

    pub fn boxed(self) -> Box<dyn Read + Send + Sync + 'static> {
        match self {
            Stream::Raw(r) => Box::new(r),
            Stream::Deflate(r) => Box::new(r),
            Stream::Gzip(r) => Box::new(r),
            Stream::Bgzf(r) => Box::new(r),
            Stream::MultithreadedBgzf(r) => Box::new(r),
        }
    }
}

impl<R: Read + Send + Sync + 'static> Read for Stream<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Stream::Raw(r) => r.read(buf),
            Stream::Deflate(r) => r.read(buf),
            Stream::Gzip(r) => r.read(buf),
            Stream::Bgzf(r) => r.read(buf),
            Stream::MultithreadedBgzf(r) => r.read(buf),
        }
    }
}
