#![allow(clippy::too_many_arguments)]

use std::fmt::Debug;

use derive_getters::Dissolve;
use derive_more::Constructor;
use itertools::Itertools;
use pyo3::prelude::*;

use biobit_core_py::loc::{PyInterval, PyOrientation};
use biobit_core_py::{
    loc::Contig,
    num::{Float, PrimInt},
};
use biobit_reaper_rs::result::Peak;
use biobit_reaper_rs::{Harvest, HarvestRegion};

#[pyclass(get_all, name = "Peak")]
#[derive(Clone, Debug, Dissolve, Constructor)]
pub struct PyPeak {
    interval: Py<PyInterval>,
    value: f64,
    summit: i64,
}

impl<Idx: PrimInt + TryInto<i64>, Cnts: Float> IntoPy<PyPeak> for Peak<Idx, Cnts> {
    fn into_py(self, py: Python<'_>) -> PyPeak {
        let (interval, value, summit) = self.dissolve();
        PyPeak::new(
            Py::new(py, interval.into_py(py)).unwrap(),
            value.to_f64().unwrap(),
            summit.to_i64().unwrap(),
        )
    }
}

#[pyclass(get_all, name = "HarvestRegion")]
#[derive(Clone, Debug, Dissolve, Constructor)]
pub struct PyHarvestRegion {
    contig: String,
    orientation: PyOrientation,
    interval: PyInterval,
    signal: Vec<Py<PyInterval>>,
    control: Vec<Py<PyInterval>>,
    modeled: Vec<Py<PyInterval>>,
    raw_peaks: Vec<Py<PyPeak>>,
    filtered_peaks: Vec<Py<PyPeak>>,
}

impl<Ctg, Idx, Cnts> IntoPy<PyHarvestRegion> for HarvestRegion<Ctg, Idx, Cnts>
where
    Ctg: Into<String> + Contig,
    Idx: PrimInt + TryInto<i64>,
    Cnts: Float,
{
    fn into_py(self, py: Python<'_>) -> PyHarvestRegion {
        let (contig, orientation, interval, signal, control, model, peaks, nms) = self.dissolve();

        let contig = contig.into();
        let orientation = orientation.into_py(py);
        let interval = interval.into_py(py);

        let (signal, control, model) = [signal, control, model]
            .into_iter()
            .map(|x| {
                x.into_iter()
                    .map(|x| Py::new(py, x.into_py(py)))
                    .collect::<PyResult<Vec<_>>>()
                    .expect("Failed to allocate Python memory for the reaper:Interval")
            })
            .collect_tuple::<(_, _, _)>()
            .unwrap();

        let (peaks, nms) = [peaks, nms]
            .into_iter()
            .map(|x| {
                x.into_iter()
                    .map(|x| Py::new(py, x.into_py(py)))
                    .collect::<PyResult<Vec<_>>>()
                    .expect("Failed to allocate Python memory for the reaper:Peak")
            })
            .collect_tuple::<(_, _)>()
            .unwrap();

        PyHarvestRegion::new(
            contig,
            orientation,
            interval,
            signal,
            control,
            model,
            peaks,
            nms,
        )
    }
}

#[pyclass(get_all, name = "Harvest")]
#[derive(Clone, Debug, Dissolve, Constructor)]
pub struct PyHarvest {
    comparison: PyObject,
    regions: Vec<Py<PyHarvestRegion>>,
}

impl<Ctg, Idx, Cnts, Tag> IntoPy<PyHarvest> for Harvest<Ctg, Idx, Cnts, Tag>
where
    Ctg: Into<String> + Contig,
    Idx: TryInto<i64> + PrimInt,
    Cnts: TryInto<f64> + Float,
    Tag: IntoPy<PyObject>,
{
    fn into_py(self, py: Python<'_>) -> PyHarvest {
        let (cmp, regions) = self.dissolve();
        let regions = regions
            .into_iter()
            .map(|x| {
                Py::new(py, x.into_py(py))
                    .expect("Failed to allocate Python memory for the ripper:Region")
            })
            .collect();

        PyHarvest::new(cmp.into_py(py), regions)
    }
}
