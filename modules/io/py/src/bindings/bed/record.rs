use biobit_core_py::loc::{Interval, Orientation};
use biobit_core_py::loc::{IntoPyInterval, IntoPyOrientation, PyInterval, PyOrientation};
use biobit_core_py::pickle;
use biobit_io_rs::bed::{
    Bed12, Bed12MutOp, Bed12Op, Bed3, Bed3MutOp, Bed3Op, Bed4, Bed4MutOp, Bed4Op, Bed5, Bed5MutOp,
    Bed5Op, Bed6, Bed6MutOp, Bed6Op, Bed8, Bed8MutOp, Bed8Op, Bed9, Bed9MutOp, Bed9Op,
};
use bitcode::{Decode, Encode};
use derive_more::{From, Into};
use eyre::{OptionExt, Result};
use paste::paste;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::PyTypeInfo;
use std::hash::{DefaultHasher, Hash, Hasher};

const U64_TO_I64_CAST_ERROR: &str =
    "Failed to cast u64 used in the BED format to i64 used in Python intervals";
const I64_TO_U64_CAST_ERROR: &str =
    "Failed to cast i64 used in Python intervals to u64 used in the BED format";

mod from_py {
    use super::*;
    pub fn not_required<T>(_py: Python, t: T) -> Result<T> {
        Ok(t)
    }

    pub fn interval(py: Python, interval: IntoPyInterval) -> Result<Interval<u64>> {
        let interval = interval.extract_rs(py).ok_or_eyre(I64_TO_U64_CAST_ERROR)?;
        Ok(interval)
    }

    pub fn orientation(_py: Python, orientation: IntoPyOrientation) -> Result<Orientation> {
        Ok(orientation.0.into())
    }

    pub fn interval_vec(py: Python, intervals: Vec<IntoPyInterval>) -> Result<Vec<Interval<u64>>> {
        let intervals: Result<Vec<_>> = intervals
            .into_iter()
            .map(|x| x.extract_rs(py).ok_or_eyre(I64_TO_U64_CAST_ERROR))
            .collect();
        intervals
    }
}

macro_rules! impl_pybed {
    (seqid $($postfix:tt)*) => {
        impl_pybed!(
            $($postfix)*
            (
                seqid: String [from_py::not_required] =>
                fn seqid(&self) -> &str {
                    self.rs.seqid()
                }

                fn set_seqid(&mut self, _py: Python, seqid: String) -> Result<()> {
                    self.rs.set_seqid(seqid)?;
                    Ok(())
                }
            )
        );
    };
    (interval $($postfix:tt)*) => {
        impl_pybed!(
            $($postfix)*
            (
                interval: IntoPyInterval [from_py::interval] =>
                fn interval(&self) -> Result<PyInterval> {
                    let interval = self
                        .rs
                        .interval()
                        .cast()
                        .ok_or_eyre(U64_TO_I64_CAST_ERROR)?
                        .into();
                    Ok(interval)
                }

                fn set_interval(&mut self, py: Python, interval: IntoPyInterval) -> Result<()> {
                    self.rs.set_interval(from_py::interval(py, interval)?)?;
                    Ok(())
                }
            )
        );
    };
    (name $($postfix:tt)*) => {
        impl_pybed!(
            $($postfix)*
            (
                name: String [from_py::not_required] =>
                fn name(&self) -> &str {
                    self.rs.name()
                }

                fn set_name(&mut self, _py: Python, name: String) -> Result<()> {
                    self.rs.set_name(name)?;
                    Ok(())
                }
            )
        );
    };
    (score $($postfix:tt)*) => {
        impl_pybed!(
            $($postfix)*
            (
                score: u16 [from_py::not_required] =>
                fn score(&self) -> u16 {
                    self.rs.score()
                }

                fn set_score(&mut self, _py: Python, score: u16) -> Result<()> {
                    self.rs.set_score(score)?;
                    Ok(())
                }
            )
        );
    };
    (orientation $($postfix:tt)*) => {
        impl_pybed!(
            $($postfix)*
            (
                orientation: IntoPyOrientation [from_py::orientation] =>
                fn orientation(&self) -> PyOrientation {
                    self.rs.orientation().into()
                }

                fn set_orientation(
                    &mut self,
                    py: Python,
                    orientation: IntoPyOrientation
                ) -> Result<()> {
                    self.rs.set_orientation(from_py::orientation(py, orientation)?)?;
                    Ok(())
                }
            )
        );
    };
    (thick $($postfix:tt)*) => {
        impl_pybed!(
            $($postfix)*
            (
                thick: IntoPyInterval [from_py::interval] =>
                fn thick(&self) -> Result<PyInterval> {
                    let thick = self
                        .rs
                        .thick()
                        .cast()
                        .ok_or_eyre(U64_TO_I64_CAST_ERROR)?
                        .into();
                    Ok(thick)
                }

                fn set_thick(&mut self, py: Python, thick: IntoPyInterval) -> Result<()> {
                    let thick = from_py::interval(py, thick)?;
                    self.rs.set_thick(thick)?;
                    Ok(())
                }
            )
        );
    };
    (rgb $($postfix:tt)*) => {
        impl_pybed!(
            $($postfix)*
            (
                rgb: (u8, u8, u8) [from_py::not_required] =>
                fn rgb(&self) -> (u8, u8, u8) {
                    self.rs.rgb()
                }

                fn set_rgb(&mut self, _py: Python, rgb: (u8, u8, u8)) -> Result<()> {
                    self.rs.set_rgb(rgb)?;
                    Ok(())
                }
            )
        );
    };
    (blocks $($postfix:tt)*) => {
        impl_pybed!(
            $($postfix)*
            (
                blocks: Vec<IntoPyInterval> [from_py::interval_vec] =>
                fn blocks(&self) -> Result<Vec<PyInterval>> {
                    let blocks = self.rs.blocks();
                    let mut pyblocks = Vec::with_capacity(self.rs.blocks().len());
                    for block in blocks {
                        let casted = (*block)
                            .cast()
                            .ok_or_eyre(U64_TO_I64_CAST_ERROR)?;

                        pyblocks.push(PyInterval::from(casted));
                    }
                    Ok(pyblocks)
                }

                fn set_blocks(&mut self, py: Python, blocks: Vec<IntoPyInterval>) -> Result<()> {
                    self.rs.set_blocks(from_py::interval_vec(py, blocks)?)?;
                    Ok(())
                }
            )
        );
    };
    (
        <$Bed:ident, $Rs:ident, $Name:literal>
        $(($field:ident: $ftype:ty [$from_py:expr] => $pyget:item $pyset:item))+
    ) => {
        paste! {
            #[pyclass(eq, ord, name = $Name)]
            #[repr(transparent)]
            #[derive(
                Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Into,
            )]
            pub struct $Bed {
                pub rs: $Rs,
            }

            #[pymethods]
            impl $Bed {
                #[new]
                #[allow(clippy::too_many_arguments)]
                fn new(py: Python, $($field: $ftype,)+) -> Result<Self> {
                    $(
                        let $field = $from_py(py, $field)?;
                    )+
                    $Rs::new($($field,)+).map(|rs| Self { rs })
                }

                #[staticmethod]
                fn default() -> Self {
                    Self {
                        rs: $Rs::default(),
                    }
                }

                $(
                    #[getter]
                    $pyget

                    #[setter]
                    $pyset
                )+


                #[pyo3(signature = ($($field=None, )+))]
                #[allow(clippy::too_many_arguments)]
                fn set(&mut self, py: Python, $($field: Option<$ftype>,)+) -> Result<()> {
                    $(
                        let $field = $field.map(|x| $from_py(py, x)).transpose()?;
                    )+
                    self.rs.set($($field,)+)?;
                    Ok(())
                }

                fn __hash__(&self) -> u64 {
                    let mut hasher = DefaultHasher::new();
                    self.hash(&mut hasher);
                    hasher.finish()
                }

                fn __repr__(&self) -> String {
                    let mut repr = String::from($Name);
                    repr.push('(');
                    $(
                        repr.push_str(&format!("{:?}, ", self.rs.$field()));
                    )+
                    repr.pop();
                    repr.pop();
                    repr.push(')');
                    repr
                }

                #[staticmethod]
                fn _from_pickle(state: &Bound<PyBytes>) -> PyResult<Self> {
                    pickle::from_bytes(state.as_bytes()).map_err(|e| e.into())
                }

                fn __reduce__(&self, py: Python) -> Result<(PyObject, (Vec<u8>,))> {
                    Ok((
                        Self::type_object(py).getattr("_from_pickle")?.unbind(),
                        (pickle::to_bytes(self),),
                    ))
                }
            }
        }
    };
}

impl_pybed!(seqid interval <PyBed3, Bed3, "Bed3">);
impl_pybed!(seqid interval name <PyBed4, Bed4, "Bed4">);
impl_pybed!(seqid interval name score <PyBed5, Bed5, "Bed5">);
impl_pybed!(seqid interval name score orientation <PyBed6, Bed6, "Bed6">);
impl_pybed!(seqid interval name score orientation thick <PyBed8, Bed8, "Bed8">);
impl_pybed!(seqid interval name score orientation thick rgb <PyBed9, Bed9, "Bed9">);
impl_pybed!(seqid interval name score orientation thick rgb blocks <PyBed12, Bed12, "Bed12">);
