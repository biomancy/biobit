//! JSON loading for the closed manifest adapter set.

use crate::Designs;
use crate::data::Data;
use crate::provenance::Provenance;
use eyre::{Result, WrapErr};
use std::fs;
use std::path::Path;

pub(super) fn load_provenance(path: &Path) -> Result<Provenance> {
    let payload = read_payload(path, "provenance")?;
    serde_json::from_slice(&payload)
        .wrap_err_with(|| format!("failed to parse provenance JSON '{}'", path.display()))
}

pub(super) fn load_data(path: &Path, provenance: &Provenance) -> Result<Data> {
    let payload = read_payload(path, "data")?;
    let mut deserializer = serde_json::Deserializer::from_slice(&payload);
    let data = Data::deserialize_with_provenance(provenance, &mut deserializer)
        .wrap_err_with(|| format!("failed to parse data JSON '{}'", path.display()))?;
    deserializer
        .end()
        .wrap_err_with(|| format!("failed to parse data JSON '{}'", path.display()))?;
    Ok(data)
}

pub(super) fn load_designs(path: &Path, provenance: &Provenance) -> Result<Designs> {
    let payload = read_payload(path, "design")?;
    let mut deserializer = serde_json::Deserializer::from_slice(&payload);
    let designs = Designs::deserialize_with_provenance(provenance, &mut deserializer)
        .wrap_err_with(|| format!("failed to parse design JSON '{}'", path.display()))?;
    deserializer
        .end()
        .wrap_err_with(|| format!("failed to parse design JSON '{}'", path.display()))?;
    Ok(designs)
}

fn read_payload(path: &Path, domain: &str) -> Result<Vec<u8>> {
    fs::read(path).wrap_err_with(|| format!("failed to read {domain} payload '{}'", path.display()))
}
