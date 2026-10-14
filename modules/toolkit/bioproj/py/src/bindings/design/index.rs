use biobit_bioproj_rs as rs;
use pyo3::prelude::*;

use super::unit::{PyUnit, PyUnitId};
use super::{DesignIdInput, DesignInput};
use crate::utils::lookup::{define_family_record_index, define_homogeneous_record_index};

define_homogeneous_record_index!(
    PyUnitIndex(rs::design::Designs) as "UnitIndex",
    record: PyUnit,
    id: PyUnitId as "UnitId",
    rust: rs::design::Unit,
    lookup: unit,
    iter: units
);

define_family_record_index!(
    PyDesignIndex(rs::design::Designs) as "DesignIndex",
    record: DesignInput,
    id: DesignIdInput as "DesignId",
    id_ref: rs::design::DesignIdRef<'_>,
    rust: rs::design::Design,
    lookup: design,
    iter: designs
);
