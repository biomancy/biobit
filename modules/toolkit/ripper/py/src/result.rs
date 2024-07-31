use std::fmt::Debug;

use derive_getters::Dissolve;
use derive_more::Constructor;
use pyo3::prelude::*;

use biobit_core_py::{
    loc::Contig,
    num::{Float, PrimInt},
};
use biobit_core_py::loc::{AsSegment, PyPerOrientation, PySegment};
use biobit_ripper_rs::result::{Peak, Region, Ripped};

#[pyclass(get_all, name = "Peak")]
#[derive(Clone, Debug, Dissolve, Constructor)]
pub struct PyPeak {
    start: i64,
    end: i64,
    value: f64,
    summit: i64,
}

impl<Idx: PrimInt, Cnts: Float> IntoPy<PyPeak> for Peak<Idx, Cnts> {
    fn into_py(self, _: Python<'_>) -> PyPeak {
        let (start, end, value, summit) = self.dissolve();
        PyPeak::new(
            start.to_i64().unwrap(),
            end.to_i64().unwrap(),
            value.to_f64().unwrap(),
            summit.to_i64().unwrap(),
        )
    }
}

#[pyclass(get_all, name = "Region")]
#[derive(Clone, Debug, Dissolve, Constructor)]
pub struct PyRegion {
    contig: String,
    segment: PySegment,
    peaks: PyPerOrientation,
}

impl<Ctg, Idx, Cnts> IntoPy<PyRegion> for Region<Ctg, Idx, Cnts>
where
    Ctg: Into<String> + Contig,
    Idx: PrimInt,
    Cnts: Float,
{
    fn into_py(self, py: Python<'_>) -> PyRegion {
        let (contig, segment, peaks) = self.dissolve();

        let contig = contig.into();
        let segment = PySegment::new(
            segment.start().to_i64().unwrap(),
            segment.end().to_i64().unwrap(),
        )
        .unwrap();

        let peaks = peaks
            .map(|_, x| {
                x.into_iter()
                    .map(|x| Py::new(py, x.into_py(py)))
                    .collect::<PyResult<Vec<_>>>()
                    .expect("Failed to allocate Python memory for the ripper:Peak")
                    .into_py(py)
            })
            .into();

        PyRegion::new(contig, segment, peaks)
    }
}

#[pyclass(get_all, name = "Ripped")]
#[derive(Clone, Debug, Dissolve, Constructor)]
pub struct PyRipped {
    tag: PyObject,
    regions: Vec<Py<PyRegion>>,
}

impl<Ctg, Idx, Cnts, Tag> IntoPy<PyRipped> for Ripped<Ctg, Idx, Cnts, Tag>
where
    Ctg: Into<String> + Contig,
    Idx: TryInto<i64> + PrimInt,
    Cnts: TryInto<f64> + Float,
    Tag: IntoPy<PyObject>,
{
    fn into_py(self, py: Python<'_>) -> PyRipped {
        let (tag, regions) = self.dissolve();
        let regions = regions
            .into_iter()
            .map(|x| {
                Py::new(py, x.into_py(py))
                    .expect("Failed to allocate Python memory for the ripper:Region")
            })
            .collect();

        PyRipped::new(tag.into_py(py), regions)
    }
}
