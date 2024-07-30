use derive_getters::Dissolve;
use derive_more::Constructor;
use pyo3::prelude::*;

use biobit_core_py::{
    loc::Contig,
    num::{Float, PrimInt},
};
use biobit_core_py::loc::{PyPerOrientation, PySegment};
use biobit_ripper_rs::result::{Peak, Region, Ripped};

#[pyclass(get_all, name = "Peak")]
#[derive(Clone, Debug, Dissolve, Constructor)]
pub struct PyPeak {
    start: i64,
    end: i64,
    value: f64,
    summit: i64,
}

impl<Idx: Into<i64>, Cnts: Into<f64>> IntoPy<PyPeak> for Peak<Idx, Cnts> {
    fn into_py(self, _: Python<'_>) -> PyPeak {
        let (start, end, value, summit) = self.dissolve();
        PyPeak::new(start.into(), end.into(), value.into(), summit.into())
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
    Idx: PrimInt + Into<i64>,
    Cnts: Into<f64> + Float,
{
    fn into_py(self, py: Python<'_>) -> PyRegion {
        let (contig, segment, peaks) = self.dissolve();

        let contig = contig.into();
        let segment = segment.cast::<i64>().into_py(py);

        let peaks = peaks
            .map(|x| {
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
    Idx: Into<i64> + PrimInt,
    Cnts: Into<f64> + Float,
    Tag: Into<PyObject>,
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

        PyRipped::new(tag.into(), regions)
    }
}
