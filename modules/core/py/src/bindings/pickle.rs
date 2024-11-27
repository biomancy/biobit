use bitcode;
use bitcode::Decode;
use eyre;
use pyo3::prelude::*;

pub fn to_bytes<T: bitcode::Encode + ?Sized>(obj: &T) -> Vec<u8> {
    bitcode::encode(obj)
}
pub fn from_bytes<'a, T: Decode<'a>>(obj: &'a [u8]) -> PyResult<T> {
    bitcode::decode(obj).map_err(|x| PyErr::from(eyre::Report::from(x)))
}
