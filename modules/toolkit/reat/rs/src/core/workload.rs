use std::collections::BTreeMap;

use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::num::PrimUInt;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Task<SeqId = String, Idx: PrimUInt = u64> {
    pub seqid: SeqId,
    pub fetch: Interval<Idx>,
    pub intervals: Vec<Interval<Idx>>,
}

impl<SeqId, Idx: PrimUInt> Task<SeqId, Idx> {
    pub fn new(seqid: SeqId, fetch: Interval<Idx>, intervals: Vec<Interval<Idx>>) -> Self {
        assert!(
            intervals.iter().all(|interval| fetch.envelops(interval)),
            "all task intervals must be inside the fetch interval"
        );

        Self {
            seqid,
            fetch,
            intervals,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Workload<SeqId = String, Idx: PrimUInt = u64> {
    pub tasks: Vec<Task<SeqId, Idx>>,
}

impl<SeqId, Idx: PrimUInt> Default for Workload<SeqId, Idx> {
    fn default() -> Self {
        Self { tasks: Vec::new() }
    }
}

impl<SeqId, Idx> Workload<SeqId, Idx>
where
    SeqId: Clone + Ord,
    Idx: PrimUInt,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_intervals(
        intervals: impl IntoIterator<Item = (SeqId, Interval<Idx>)>,
        max_task_size: Idx,
    ) -> Self {
        assert!(max_task_size > Idx::zero(), "max_task_size must be > 0");

        let mut by_seqid: BTreeMap<SeqId, Vec<Interval<Idx>>> = BTreeMap::new();
        for (seqid, interval) in intervals {
            by_seqid.entry(seqid).or_default().push(interval);
        }

        let tasks = by_seqid
            .into_iter()
            .flat_map(|(seqid, mut intervals)| {
                let merged = Interval::merge(&mut intervals);
                let chunks = split_intervals(merged.into_iter(), max_task_size);
                pack(seqid, chunks, max_task_size)
            })
            .collect();

        Self { tasks }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

fn split_intervals<Idx: PrimUInt>(
    intervals: impl IntoIterator<Item = Interval<Idx>>,
    max_len: Idx,
) -> Vec<Interval<Idx>> {
    intervals
        .into_iter()
        .flat_map(|interval| split_interval(interval, max_len))
        .collect()
}

fn split_interval<Idx: PrimUInt>(interval: Interval<Idx>, max_len: Idx) -> Vec<Interval<Idx>> {
    debug_assert!(max_len > Idx::zero());

    let mut chunks = Vec::new();
    let mut start = interval.start();
    while start < interval.end() {
        let step = (interval.end() - start).min(max_len);
        let end = start + step;
        chunks.push(Interval::new(start, end).unwrap());
        start = end;
    }
    chunks
}

fn pack<SeqId, Idx: PrimUInt>(
    seqid: SeqId,
    mut intervals: Vec<Interval<Idx>>,
    max_task_size: Idx,
) -> Vec<Task<SeqId, Idx>>
where
    SeqId: Clone,
{
    if intervals.is_empty() {
        return Vec::new();
    }
    intervals.sort();

    let mut tasks = Vec::new();
    let mut iter = intervals.into_iter();
    let first = iter.next().unwrap();
    let mut fetch_start = first.start();
    let mut fetch_limit = fetch_start
        .checked_add(&max_task_size)
        .unwrap_or_else(Idx::max_value);
    let mut fetch_end = first.end();
    let mut buffer = vec![first];

    for interval in iter {
        if interval.end() > fetch_limit {
            tasks.push(task(seqid.clone(), fetch_start, fetch_end, buffer));

            fetch_start = interval.start();
            fetch_limit = fetch_start
                .checked_add(&max_task_size)
                .unwrap_or_else(Idx::max_value);
            fetch_end = interval.end();
            buffer = vec![interval];
        } else {
            fetch_end = fetch_end.max(interval.end());
            buffer.push(interval);
        }
    }

    tasks.push(task(seqid, fetch_start, fetch_end, buffer));
    tasks
}

fn task<SeqId, Idx: PrimUInt>(
    seqid: SeqId,
    fetch_start: Idx,
    fetch_end: Idx,
    intervals: Vec<Interval<Idx>>,
) -> Task<SeqId, Idx> {
    Task::new(
        seqid,
        Interval::new(fetch_start, fetch_end).unwrap(),
        intervals,
    )
}

#[cfg(test)]
mod tests {
    use eyre::Result;

    use super::*;

    fn interval(start: u64, end: u64) -> Interval<u64> {
        Interval::new(start, end).unwrap()
    }

    #[test]
    fn builds_empty_workload() {
        let workload = Workload::<String, u64>::from_intervals([], 100);
        assert!(workload.is_empty());
        assert_eq!(workload.len(), 0);
    }

    #[test]
    fn groups_intervals_by_seqid() -> Result<()> {
        let workload = Workload::from_intervals(
            [
                ("chr2".to_string(), interval(0, 10)),
                ("chr1".to_string(), interval(10, 20)),
                ("chr1".to_string(), interval(30, 40)),
            ],
            50,
        );

        assert_eq!(
            workload.tasks,
            vec![
                Task::new(
                    "chr1".to_string(),
                    interval(10, 40),
                    vec![interval(10, 20), interval(30, 40)]
                ),
                Task::new("chr2".to_string(), interval(0, 10), vec![interval(0, 10)]),
            ]
        );
        Ok(())
    }

    #[test]
    fn splits_large_intervals() {
        let workload = Workload::from_intervals([("chr1".to_string(), interval(0, 284))], 100);

        assert_eq!(
            workload.tasks,
            vec![
                Task::new("chr1".to_string(), interval(0, 100), vec![interval(0, 100)]),
                Task::new(
                    "chr1".to_string(),
                    interval(100, 200),
                    vec![interval(100, 200)]
                ),
                Task::new(
                    "chr1".to_string(),
                    interval(200, 284),
                    vec![interval(200, 284)]
                ),
            ]
        );
    }

    #[test]
    fn merges_overlapping_requested_intervals() {
        let workload = Workload::from_intervals(
            [
                ("chr1".to_string(), interval(0, 10)),
                ("chr1".to_string(), interval(5, 20)),
                ("chr1".to_string(), interval(20, 30)),
            ],
            100,
        );

        assert_eq!(
            workload.tasks,
            vec![Task::new(
                "chr1".to_string(),
                interval(0, 30),
                vec![interval(0, 30)]
            )]
        );
    }

    #[test]
    fn starts_new_task_when_next_interval_exceeds_fetch_limit() {
        let workload = Workload::from_intervals(
            [
                ("chr1".to_string(), interval(0, 3)),
                ("chr1".to_string(), interval(2, 5)),
                ("chr1".to_string(), interval(4, 8)),
            ],
            5,
        );

        assert_eq!(
            workload.tasks,
            vec![
                Task::new("chr1".to_string(), interval(0, 5), vec![interval(0, 5)]),
                Task::new("chr1".to_string(), interval(5, 8), vec![interval(5, 8)]),
            ]
        );
    }

    #[test]
    fn supports_non_default_coordinate_type() -> Result<()> {
        let workload =
            Workload::from_intervals([("chr1".to_string(), Interval::new(0_u32, 10_u32)?)], 5_u32);

        let _: Interval<u32> = workload.tasks[0].fetch;
        assert_eq!(workload.len(), 2);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn task_requires_intervals_inside_fetch() {
        Task::new("chr1", interval(10, 20), vec![interval(0, 5)]);
    }
}
