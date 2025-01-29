#![allow(clippy::too_many_arguments)]

use std::fmt::Debug;

use derive_getters::Dissolve;
use derive_more::Constructor;
use itertools::Itertools;
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;

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

impl<Idx: PrimInt + TryInto<i64>, Cnts: Float> From<Peak<Idx, Cnts>> for PyPeak {
    fn from(peak: Peak<Idx, Cnts>) -> Self {
        let (interval, value, summit) = peak.dissolve();
        let interval = interval.cast::<i64>().unwrap();
        Python::with_gil(|py| {
            PyPeak::new(
                Py::new(py, PyInterval::from(interval)).unwrap(),
                value.to_f64().unwrap(),
                summit.to_i64().unwrap(),
            )
        })
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

impl<Ctg, Idx, Cnts> From<HarvestRegion<Ctg, Idx, Cnts>> for PyHarvestRegion
where
    Ctg: Into<String> + Contig,
    Idx: PrimInt + TryInto<i64>,
    Cnts: Float,
{
    fn from(value: HarvestRegion<Ctg, Idx, Cnts>) -> Self {
        let (contig, orientation, interval, signal, control, model, peaks, nms) = value.dissolve();

        Python::with_gil(|py| {
            let contig = contig.into();
            let orientation = orientation.into();
            let interval = interval.cast::<i64>().unwrap().into();

            let (signal, control, model) = [signal, control, model]
                .into_iter()
                .map(|x| {
                    x.into_iter()
                        .map(|x| x.cast::<i64>().unwrap())
                        .map(|x| Py::new(py, PyInterval::from(x)))
                        .collect::<PyResult<Vec<_>>>()
                        .expect("Failed to allocate Python memory for the reaper:Interval")
                })
                .collect_tuple::<(_, _, _)>()
                .unwrap();

            let (peaks, nms) = [peaks, nms]
                .into_iter()
                .map(|x| {
                    x.into_iter()
                        .map(|x| Py::new(py, PyPeak::from(x)))
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
        })
    }
}

#[pyclass(get_all, name = "Harvest")]
#[derive(Clone, Debug, Dissolve, Constructor)]
pub struct PyHarvest {
    comparison: PyObject,
    regions: Vec<Py<PyHarvestRegion>>,
}

impl<Ctg, Idx, Cnts, Tag> From<Harvest<Ctg, Idx, Cnts, Tag>> for PyHarvest
where
    Ctg: Into<String> + Contig,
    Idx: TryInto<i64> + PrimInt,
    Cnts: TryInto<f64> + Float,
    Tag: for<'a> IntoPyObject<'a>,
{
    fn from(value: Harvest<Ctg, Idx, Cnts, Tag>) -> Self {
        let (cmp, regions) = value.dissolve();
        Python::with_gil(|py| {
            let regions = regions
                .into_iter()
                .map(|x| {
                    Py::new(py, PyHarvestRegion::from(x))
                        .expect("Failed to allocate Python memory for the ripper:Region")
                })
                .collect();

            PyHarvest::new(cmp.into_py_any(py).unwrap(), regions)
        })
    }
}
