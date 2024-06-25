use std::collections::HashMap;

use biobit_core_rs::loc::{Contig, Locus};
use biobit_core_rs::num::{Float, PrimInt};
use biobit_core_rs::traits::reads_counter;

pub struct Stats<Ctg: Contig, Idx: PrimInt> {
    time_s: f64,
    partition: Locus<Ctg, Idx>,
    inside_annotation: f64,
    outside_annotation: f64,
}

pub struct Counts<Src, Data, Cnts: Float, Ctg: Contig, Idx: PrimInt> {
    source: Src,
    counts: HashMap<Data, Cnts>,
    stats: Vec<Stats<Ctg, Idx>>,
}

impl<Src, Data, Cnts: Float, Ctg: Contig, Idx: PrimInt> reads_counter::Result for Counts<Src, Data, Cnts, Ctg, Idx> {

}