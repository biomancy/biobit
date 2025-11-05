use crate::library::Library;
use crate::sample::Sample;
use serde::Serializer;

pub fn library_ind<S: Serializer>(lib: &Library, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(lib.ind())
}

pub fn sample_ind<S: Serializer>(sample: &Sample, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(sample.ind())
}
