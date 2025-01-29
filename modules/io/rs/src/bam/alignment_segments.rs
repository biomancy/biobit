use biobit_core_rs::loc::Interval;
use biobit_core_rs::num::PrimInt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct AlignmentSegments<Idx: PrimInt> {
    /// Ranges of the alignment segments [start, end), many segments correspond to a single alignment
    segments: Vec<Interval<Idx>>,
    /// Each i-th alignment corresponds to segments[alignments[i]..alignments[i + 1]]
    alignments: Vec<usize>,
}

impl<Idx: PrimInt> AlignmentSegments<Idx> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            segments: Vec::with_capacity(capacity),
            alignments: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.alignments.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.alignments.is_empty()
    }

    pub fn len(&self) -> usize {
        if self.alignments.is_empty() {
            0
        } else {
            self.alignments.len() - 1
        }
    }

    pub fn push(&mut self, segments: &[Interval<Idx>]) {
        if segments.is_empty() {
            return;
        }

        if self.alignments.is_empty() {
            self.alignments.push(0);
        }
        self.segments.extend_from_slice(segments);
        self.alignments.push(self.segments.len());
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ [Interval<Idx>]> {
        (0..self.len()).map(move |i| &self.segments[self.alignments[i]..self.alignments[i + 1]])
    }

    pub fn at(&self, i: usize) -> &[Interval<Idx>] {
        &self.segments[self.alignments[i]..self.alignments[i + 1]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alignment_segments() {
        let mut segments = AlignmentSegments::<u32>::with_capacity(10);
        assert_eq!(segments.len(), 0);
        assert_eq!(segments.iter().count(), 0);

        segments.push(&[(0, 10).try_into().unwrap(), (20, 30).try_into().unwrap()]);
        assert_eq!(segments.len(), 1);

        segments.push(&[(5, 12).try_into().unwrap(), (200, 300).try_into().unwrap()]);
        assert_eq!(segments.len(), 2);

        segments.push(&[(0, 1).try_into().unwrap()]);
        assert_eq!(segments.len(), 3);

        let mut iter = segments.iter();
        for (ind, elements) in [vec![0..10, 20..30], vec![5..12, 200..300], vec![0..1]]
            .iter()
            .enumerate()
        {
            let alnblocks = iter.next().unwrap();
            assert_eq!(elements, alnblocks);

            let alnblocks = segments.at(ind);
            assert_eq!(elements, alnblocks);
        }
        assert_eq!(iter.next(), None);
    }
}
