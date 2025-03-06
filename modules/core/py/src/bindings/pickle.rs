use bitcode;
use bitcode::Decode;
use eyre::Result;

pub fn to_bytes<T: bitcode::Encode + ?Sized>(obj: &T) -> Vec<u8> {
    bitcode::encode(obj)
}
pub fn from_bytes<'a, T: Decode<'a>>(obj: &'a [u8]) -> Result<T> {
    bitcode::decode(obj).map_err(eyre::Report::from)
}
