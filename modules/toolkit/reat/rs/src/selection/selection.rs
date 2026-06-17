use bitvec::{
    bitvec,
    prelude::{BitVec, Lsb0},
};

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Selection {
    selected: BitVec<usize, Lsb0>,
}

impl Selection {
    pub fn zeros(len: usize) -> Self {
        Self {
            selected: bitvec![usize, Lsb0; 0; len],
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.selected.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    #[inline]
    pub fn is_selected(&self, offset: usize) -> bool {
        debug_assert!(offset < self.len(), "selection offset is out of bounds");
        self.selected[offset]
    }

    #[inline]
    pub fn select(&mut self, offset: usize) {
        debug_assert!(offset < self.len(), "selection offset is out of bounds");
        self.selected.set(offset, true);
    }

    #[inline]
    pub fn exclude(&mut self, offset: usize) {
        debug_assert!(offset < self.len(), "selection offset is out of bounds");
        self.selected.set(offset, false);
    }

    #[inline]
    pub fn reset(&mut self, len: usize) {
        self.selected.resize(len, false);
        self.selected.fill(false);
    }

    pub fn selected_offsets(&self) -> impl Iterator<Item = usize> + '_ {
        self.selected.iter_ones()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resets_and_reports_offsets() {
        let mut selection = Selection::zeros(2);
        selection.select(0);
        selection.select(1);
        selection.exclude(0);
        assert!(!selection.is_selected(0));
        assert!(selection.is_selected(1));
        assert_eq!(selection.selected_offsets().collect::<Vec<_>>(), vec![1]);

        selection.reset(3);
        assert_eq!(selection.len(), 3);
        assert!(selection.selected_offsets().next().is_none());
        assert!(!selection.is_selected(2));
    }

    #[test]
    fn uses_packed_bitvec_storage() {
        let mut selection = Selection::zeros(66);
        assert_eq!(selection.len(), 66);
        assert_eq!(selection.selected.as_raw_slice().len(), 2);

        selection.select(0);
        selection.select(63);
        selection.select(64);
        selection.select(65);

        assert_eq!(
            selection.selected_offsets().collect::<Vec<_>>(),
            vec![0, 63, 64, 65]
        );

        selection.exclude(63);
        assert_eq!(
            selection.selected_offsets().collect::<Vec<_>>(),
            vec![0, 64, 65]
        );
    }
}
