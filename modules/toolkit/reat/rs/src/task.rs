use std::collections::BTreeMap;

use biobit_core_rs::loc::{Interval, IntervalOp};
use biobit_core_rs::num::PrimUInt;
#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use derive_getters::Getters;
use eyre::{Result, ensure, eyre};

use crate::selection::Selection;

#[cfg_attr(feature = "bitcode", derive(Encode, Decode))]
#[derive(Clone, PartialEq, Eq, Debug, Getters)]
pub struct Task<SeqId = String, Idx: PrimUInt = u64> {
    seqid: SeqId,
    envelope: Interval<Idx>,
    intervals: Vec<Interval<Idx>>,
}

impl<SeqId, Idx: PrimUInt> Task<SeqId, Idx> {
    pub fn new(seqid: SeqId, intervals: Vec<Interval<Idx>>) -> Result<Self> {
        ensure!(
            !intervals.is_empty(),
            "Task must have at least one interval"
        );

        // Ensure that intervals are non-overlapping and sorted
        let (start, mut end) = intervals[0].into();
        for it in intervals[1..].iter() {
            if it.start() < end {
                return Err(eyre::eyre!(
                    "Task intervals must be non-overlapping and sorted"
                ));
            }
            end = it.end();
        }
        let envelope = Interval::new(start, end)?;

        Ok(Self {
            seqid,
            envelope,
            intervals,
        })
    }

    pub fn exclude_outside_intervals(&self, selection: &mut Selection) -> Result<()> {
        let envelope = self
            .envelope
            .cast::<usize>()
            .ok_or_else(|| eyre!("Task envelope coordinates do not fit into usize"))?;
        ensure!(
            selection.len() == envelope.len(),
            "selection length does not match task envelope length"
        );

        // We can save 1 iteration here, because envelope.start == intervals[0].start
        // But that makes code slightly harder to read
        let mut offset = 0;
        for interval in &self.intervals {
            let interval = interval
                .cast::<usize>()
                .ok_or_else(|| eyre::eyre!("Interval coordinates do not fit into usize"))?;

            for excluded in offset..(interval.start() - envelope.start()) {
                selection.exclude(excluded);
            }
            offset = interval.end() - envelope.start();
        }

        for excluded in offset..envelope.len() {
            selection.exclude(excluded);
        }
        Ok(())
    }

    pub fn from_intervals(
        intervals: impl IntoIterator<Item = (SeqId, Interval<Idx>)>,
        max_task_size: Idx,
    ) -> Result<Vec<Self>>
    where
        SeqId: Clone + Ord,
    {
        ensure!(max_task_size > Idx::zero(), "max_task_size must be > 0");

        let mut by_seqid: BTreeMap<SeqId, Vec<Interval<Idx>>> = BTreeMap::new();
        for (seqid, interval) in intervals {
            by_seqid.entry(seqid).or_default().push(interval);
        }

        let mut tasks = Vec::new();
        for (seqid, mut intervals) in by_seqid {
            build_tasks(seqid, &mut intervals, max_task_size, &mut tasks)?;
        }
        Ok(tasks)
    }
}

fn build_tasks<SeqId: Clone, Idx: PrimUInt>(
    seqid: SeqId,
    intervals: &mut [Interval<Idx>],
    max_task_size: Idx,
    saveto: &mut Vec<Task<SeqId, Idx>>,
) -> Result<()> {
    debug_assert!(max_task_size > Idx::zero());

    let mut buffer = Vec::new();
    let mut fetch_limit = Idx::zero();

    for interval in Interval::merge(intervals) {
        let mut start = interval.start();
        while start < interval.end() {
            let step = (interval.end() - start).min(max_task_size);
            let end = start + step;
            let chunk = Interval::new(start, end)?;

            if !buffer.is_empty() && chunk.end() > fetch_limit {
                saveto.push(Task::new(seqid.clone(), buffer)?);
                buffer = Vec::new();
            }
            if buffer.is_empty() {
                fetch_limit = chunk
                    .start()
                    .checked_add(&max_task_size)
                    .unwrap_or_else(Idx::max_value);
            }

            buffer.push(chunk);
            start = end;
        }
    }

    if !buffer.is_empty() {
        saveto.push(Task::new(seqid, buffer)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use eyre::Result;

    use super::*;

    fn interval(start: u64, end: u64) -> Interval<u64> {
        Interval::new(start, end).unwrap()
    }

    #[test]
    fn builds_empty_tasks() -> Result<()> {
        let tasks: Vec<Task<String, u64>> = Task::from_intervals([], 100)?;
        assert!(tasks.is_empty());
        assert_eq!(tasks.len(), 0);
        Ok(())
    }

    #[test]
    fn groups_intervals_by_seqid() -> Result<()> {
        let tasks = Task::from_intervals(
            [
                ("chr2".to_string(), interval(0, 10)),
                ("chr1".to_string(), interval(10, 20)),
                ("chr1".to_string(), interval(30, 40)),
            ],
            50,
        )?;

        assert_eq!(
            tasks,
            vec![
                Task::new("chr1".to_string(), vec![interval(10, 20), interval(30, 40)])?,
                Task::new("chr2".to_string(), vec![interval(0, 10)])?,
            ]
        );
        Ok(())
    }

    #[test]
    fn splits_large_intervals() -> Result<()> {
        let tasks = Task::from_intervals([("chr1".to_string(), interval(0, 284))], 100)?;

        assert_eq!(
            tasks,
            vec![
                Task::new("chr1".to_string(), vec![interval(0, 100)])?,
                Task::new("chr1".to_string(), vec![interval(100, 200)])?,
                Task::new("chr1".to_string(), vec![interval(200, 284)])?,
            ]
        );
        Ok(())
    }

    #[test]
    fn merges_overlapping_requested_intervals() -> Result<()> {
        let tasks = Task::from_intervals(
            [
                ("chr1".to_string(), interval(0, 10)),
                ("chr1".to_string(), interval(5, 20)),
                ("chr1".to_string(), interval(20, 30)),
            ],
            100,
        )?;

        assert_eq!(
            tasks,
            vec![Task::new("chr1".to_string(), vec![interval(0, 30)])?]
        );
        Ok(())
    }

    #[test]
    fn starts_new_task_when_next_interval_exceeds_fetch_limit() -> Result<()> {
        let tasks = Task::from_intervals(
            [
                ("chr1".to_string(), interval(0, 3)),
                ("chr1".to_string(), interval(2, 5)),
                ("chr1".to_string(), interval(4, 8)),
            ],
            5,
        )?;

        assert_eq!(
            tasks,
            vec![
                Task::new("chr1".to_string(), vec![interval(0, 5)])?,
                Task::new("chr1".to_string(), vec![interval(5, 8)])?,
            ]
        );
        Ok(())
    }

    #[test]
    fn starts_new_task_when_gap_exceeds_fetch_limit() -> Result<()> {
        let tasks = Task::from_intervals(
            [
                ("chr1".to_string(), interval(0, 10)),
                ("chr1".to_string(), interval(100, 110)),
            ],
            20,
        )?;

        assert_eq!(
            tasks,
            vec![
                Task::new("chr1".to_string(), vec![interval(0, 10)])?,
                Task::new("chr1".to_string(), vec![interval(100, 110)])?,
            ]
        );
        Ok(())
    }

    #[test]
    fn task_excludes_selection_outside_intervals() -> Result<()> {
        let task = Task::new("chr1", vec![interval(10, 12), interval(15, 20)])?;
        let mut selection = Selection::zeros(10);
        for offset in 0..selection.len() {
            selection.select(offset);
        }

        task.exclude_outside_intervals(&mut selection)?;

        assert_eq!(
            selection.selected_offsets().collect::<Vec<_>>(),
            vec![0, 1, 5, 6, 7, 8, 9]
        );
        Ok(())
    }

    #[test]
    fn task_rejects_selection_with_wrong_length() -> Result<()> {
        let task = Task::new("chr1", vec![interval(12, 16)])?;
        let mut selection = Selection::zeros(9);

        assert!(task.exclude_outside_intervals(&mut selection).is_err());
        Ok(())
    }

    #[test]
    fn rejects_max_task_size_zero() {
        assert!(Task::from_intervals([("chr1", interval(0, 10))], 0).is_err());
    }

    #[test]
    fn task_rejects_empty_intervals() {
        assert!(Task::<&str, u64>::new("chr1", vec![]).is_err());
    }
}
