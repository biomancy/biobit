use std::borrow::Borrow;

use biobit_bioproj_rs as rs;
use pyo3::prelude::*;

use crate::data::PyData;
use crate::design::PyDesigns;
use crate::primitives::{PyMeta, define_id_binding};
use crate::provenance::PyProvenance;
use crate::utils::error::PyBioprojResult;
use crate::utils::frozen::{Frozen, frozen_wrapper};

define_id_binding!(PyProjectId, rs::ProjectId, "ProjectId");

#[pyclass(from_py_object, frozen, eq, name = "Project")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyProject(pub Frozen<rs::Project>);
frozen_wrapper!(PyProject, rs::Project);
crate::utils::pickle::pickle_methods!(PyProject, rs::Project, |value| value.0.as_ref());

#[pymethods]
impl PyProject {
    #[new]
    #[pyo3(signature = (id, *, provenance, data, designs, meta=None, description=None))]
    fn new(
        id: PyProjectId,
        provenance: PyRef<'_, PyProvenance>,
        data: PyRef<'_, PyData>,
        designs: PyRef<'_, PyDesigns>,
        meta: Option<PyMeta>,
        description: Option<String>,
    ) -> PyBioprojResult<Self> {
        Ok(rs::Project::new(
            id.0.as_ref().clone(),
            provenance.0.as_ref().clone(),
            Borrow::<rs::data::Data>::borrow(&*data).clone(),
            Borrow::<rs::design::Designs>::borrow(&*designs).clone(),
            PyMeta::option_into_meta(meta),
            description.map(rs::Text::new).transpose()?,
        )?
        .into())
    }

    #[getter]
    fn id(&self) -> PyProjectId {
        self.0.map(rs::Project::id).into()
    }

    #[getter]
    fn provenance(&self) -> PyProvenance {
        self.0.map(rs::Project::provenance).into()
    }

    #[getter]
    fn data(&self) -> PyData {
        self.0.map(rs::Project::data).into()
    }

    #[getter]
    fn designs(&self) -> PyDesigns {
        self.0.map(rs::Project::designs).into()
    }

    #[getter]
    fn meta(&self) -> PyMeta {
        self.0.map(rs::Project::meta).into()
    }

    #[getter]
    fn description(&self) -> Option<&str> {
        self.0.as_ref().description().map(|value| value.as_str())
    }
}
