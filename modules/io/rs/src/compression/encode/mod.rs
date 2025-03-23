mod algorithm;
mod config;
pub mod params;
mod stream;

pub use algorithm::Algorithm;
pub use config::Config;
pub use stream::Stream;

pub fn infer_from_path(path: impl AsRef<std::path::Path>) -> eyre::Result<Stream<std::fs::File>> {
    let path = path.as_ref();
    Stream::new(std::fs::File::open(path)?, &Config::infer_from_path(path))
}
