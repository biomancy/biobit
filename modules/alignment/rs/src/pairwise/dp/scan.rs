use crate::Alignable;

use super::layout::{AboveAntiDiagonal, AboveMainDiagonal, FullMatrix, LayoutError};

/// Two sequences and the region of their alignment matrix to visit.
///
/// The first sequence defines the rows and the second defines the columns.
/// Pass slices or shared references to borrow the sequences, or pass a view such as
/// [`Reversed`](crate::alignable::Reversed) by value. Each constructor sizes the
/// layout from those sequences and propagates any [`LayoutError`]. A scan can be
/// reused across kernels and engines while its sequence lengths stay the same.
#[derive(Debug)]
pub struct Scan<S1, S2, L> {
    seq1: S1,
    seq2: S2,
    layout: L,
}

impl<S1: Alignable, S2: Alignable<Symbol = S1::Symbol>> Scan<S1, S2, FullMatrix> {
    /// Visits the entire matrix.
    pub fn full(seq1: S1, seq2: S2) -> Result<Self, LayoutError> {
        let layout = FullMatrix::new(seq1.len(), seq2.len())?;
        Ok(Self { seq1, seq2, layout })
    }
}

impl<S1: Alignable, S2: Alignable<Symbol = S1::Symbol>> Scan<S1, S2, AboveMainDiagonal> {
    /// Visits cells satisfying `row + offset <= col`. Zero includes the main diagonal.
    /// [`AboveMainDiagonal::new`] checks that the entire triangle fits.
    pub fn above_main_diagonal(seq1: S1, seq2: S2, offset: usize) -> Result<Self, LayoutError> {
        let layout = AboveMainDiagonal::new(seq1.len(), seq2.len(), offset)?;
        Ok(Self { seq1, seq2, layout })
    }
}

impl<S1: Alignable, S2: Alignable<Symbol = S1::Symbol>> Scan<S1, S2, AboveAntiDiagonal> {
    /// Visits cells satisfying `row + col + offset < seq1.len()`. Zero includes the anti-diagonal.
    /// [`AboveAntiDiagonal::new`] checks that the entire triangle fits.
    pub fn above_anti_diagonal(seq1: S1, seq2: S2, offset: usize) -> Result<Self, LayoutError> {
        let layout = AboveAntiDiagonal::new(seq1.len(), seq2.len(), offset)?;
        Ok(Self { seq1, seq2, layout })
    }
}

impl<S1, S2, L> Scan<S1, S2, L> {
    /// The row and column sequences, in that order.
    pub fn sequences(&self) -> (&S1, &S2) {
        (&self.seq1, &self.seq2)
    }

    /// The layout used by the kernel and by subsequent traceback queries.
    pub fn layout(&self) -> &L {
        &self.layout
    }
}
