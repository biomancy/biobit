//! Design records, indexes, and Python module registration.

mod group_contrast;
mod index;
mod matched_pairs;
mod unit;
mod unit_contrast;
mod unit_set;

use biobit_bioproj_rs as rs;
use biobit_core_py::utils::ImportablePyModuleBuilder;
use pyo3::prelude::*;
use pyo3::types::PyMapping;

use crate::primitives::define_family_conversions;
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};
use biobit_core_py::utils::collections::Iterable;

use rs::design::DesignIdRef;

use group_contrast::{PyGroupContrast, PyGroupContrastId};
use index::{PyDesignIndex, PyUnitIndex};
use matched_pairs::{PyMatchedPairs, PyMatchedPairsId};
use unit::{PyUnit, PyUnitId};
use unit_contrast::{PyUnitContrast, PyUnitContrastId};
use unit_set::{PyUnitSet, PyUnitSetId};

define_family_conversions! {
    DesignInput, DesignIdInput {
        rust: rs::design::Design,
        id: rs::design::DesignId,
        id_ref: DesignIdRef,
        variants: {
            UnitContrast => (PyUnitContrast, PyUnitContrastId),
            GroupContrast => (PyGroupContrast, PyGroupContrastId),
            MatchedPairs => (PyMatchedPairs, PyMatchedPairsId),
            UnitSet => (PyUnitSet, PyUnitSetId),
        }
    }
}

#[pyclass(from_py_object, frozen, eq, name = "Designs")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyDesigns(pub Frozen<rs::design::Designs>);
frozen_wrapper!(PyDesigns, rs::design::Designs);

#[pymethods]
impl PyDesigns {
    #[new]
    #[pyo3(signature = (*, units, designs))]
    fn new(units: Iterable<PyUnit>, designs: Iterable<DesignInput>) -> PyBioprojResult<Self> {
        Ok(rs::design::Designs::new(
            units.into_iter().map(|value| value.0.as_ref().clone()),
            designs.into_iter().map(Into::into),
        )?
        .into())
    }

    #[getter]
    fn units(&self) -> PyUnitIndex {
        PyUnitIndex(self.0.clone())
    }

    #[getter]
    fn designs(&self) -> PyDesignIndex {
        PyDesignIndex(self.0.clone())
    }
}

pub fn construct<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyModule>> {
    PyMapping::register::<PyUnitIndex>(py)?;
    PyMapping::register::<PyDesignIndex>(py)?;
    let module = ImportablePyModuleBuilder::new(py, name)?
        .defaults()?
        .add_class::<PyUnitId>()?
        .add_class::<PyUnit>()?
        .add_class::<PyUnitContrastId>()?
        .add_class::<PyUnitContrast>()?
        .add_class::<PyGroupContrastId>()?
        .add_class::<PyGroupContrast>()?
        .add_class::<PyMatchedPairsId>()?
        .add_class::<PyMatchedPairs>()?
        .add_class::<PyUnitSetId>()?
        .add_class::<PyUnitSet>()?
        .add_class::<PyDesigns>()?
        .add_class::<PyUnitIndex>()?
        .add_class::<PyDesignIndex>()?
        .finish();
    module.add(
        "Design",
        module
            .getattr("UnitContrast")?
            .bitor(module.getattr("GroupContrast")?)?
            .bitor(module.getattr("MatchedPairs")?)?
            .bitor(module.getattr("UnitSet")?)?,
    )?;
    module.add(
        "DesignId",
        module
            .getattr("UnitContrastId")?
            .bitor(module.getattr("GroupContrastId")?)?
            .bitor(module.getattr("MatchedPairsId")?)?
            .bitor(module.getattr("UnitSetId")?)?,
    )?;
    module.add(
        "__all__",
        [
            "Design",
            "DesignId",
            "DesignIndex",
            "Designs",
            "GroupContrast",
            "GroupContrastId",
            "MatchedPairs",
            "MatchedPairsId",
            "Unit",
            "UnitContrast",
            "UnitContrastId",
            "UnitId",
            "UnitIndex",
            "UnitSet",
            "UnitSetId",
        ],
    )?;
    Ok(module)
}
