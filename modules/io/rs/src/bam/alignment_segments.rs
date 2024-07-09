use biobit_core_rs::loc::{Orientation, Segment};
use biobit_core_rs::num::PrimInt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct AlignmentSegments<Idx: PrimInt> {
    /// Ranges of the alignment segments [start, end), many segments correspond to a single alignment
    segments: Vec<Segment<Idx>>,
    /// Each i-th alignment corresponds to segments[alignments[i]..alignments[i + 1]]
    alignments: Vec<usize>,
    /// orientation for i-th alignment
    orientation: Vec<Orientation>,
}

impl<Idx: PrimInt> AlignmentSegments<Idx> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            segments: Vec::with_capacity(capacity),
            alignments: Vec::with_capacity(capacity),
            orientation: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.alignments.clear();
        self.orientation.clear();
    }

    pub fn len(&self) -> usize {
        self.orientation.len()
    }

    pub fn push(&mut self, segments: &[Segment<Idx>], orientation: Orientation) {
        if segments.is_empty() {
            return;
        }

        if self.alignments.is_empty() {
            self.alignments.push(0);
        }
        self.segments.extend_from_slice(segments);
        self.alignments.push(self.segments.len());
        self.orientation.push(orientation);
        debug_assert_eq!(
            self.alignments.len(),
            self.orientation.len() + 1,
            "AlignmentSegments invariant violated"
        );
    }

    pub fn iter(&self) -> impl Iterator<Item=(&'_ [Segment<Idx>], Orientation)> {
        self.orientation
            .iter()
            .enumerate()
            .map(|(i, &orientation)| {
                (
                    &self.segments[self.alignments[i]..self.alignments[i + 1]],
                    orientation,
                )
            })
    }

    pub fn at(&self, i: usize) -> (&[Segment<Idx>], Orientation) {
        (
            &self.segments[self.alignments[i]..self.alignments[i + 1]],
            self.orientation[i],
        )
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_alignment_segments() {
        let mut segments = AlignmentSegments::<u32>::with_capacity(10);
        assert_eq!(segments.len(), 0);
        assert_eq!(segments.iter().count(), 0);

        segments.push(
            &[(0, 10).try_into().unwrap(), (20, 30).try_into().unwrap()],
            Orientation::Forward,
        );
        assert_eq!(segments.len(), 1);

        segments.push(
            &[(5, 12).try_into().unwrap(), (200, 300).try_into().unwrap()],
            Orientation::Reverse,
        );
        assert_eq!(segments.len(), 2);

        segments.push(&[(0, 1).try_into().unwrap()], Orientation::Dual);
        assert_eq!(segments.len(), 3);

        let mut iter = segments.iter();
        for (ind, (elements, orientation)) in [
            (vec![0..10, 20..30], Orientation::Forward),
            (vec![5..12, 200..300], Orientation::Reverse),
            (vec![0..1], Orientation::Dual),
        ]
            .iter()
            .enumerate()
        {
            let alnblocks = iter.next().unwrap();
            assert_eq!(elements, alnblocks.0);
            assert_eq!(*orientation, alnblocks.1);

            let alnblocks = segments.at(ind);
            assert_eq!(elements, alnblocks.0);
            assert_eq!(*orientation, alnblocks.1);
        }
        assert_eq!(iter.next(), None);
    }
}
