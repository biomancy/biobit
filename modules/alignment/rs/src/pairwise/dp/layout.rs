use std::ops::Range;

/// Errors returned when constructing an alignment layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq, derive_more::Display, derive_more::Error)]
pub enum LayoutError {
    /// The offset exceeds the length of the axis that defines the triangle.
    #[display("diagonal offset {offset} is outside an axis of length {axis_len}")]
    OffsetOutOfBounds { offset: usize, axis_len: usize },
    /// The other sequence is too short for the requested triangle.
    #[display("triangle side {side} exceeds the other sequence's length {available}")]
    TriangleDoesNotFit { side: usize, available: usize },
    /// The number of cells in the layout exceeds the dense index capacity.
    #[display("layout has {cells} cells, which exceeds usize::MAX")]
    CellCountOverflow { cells: u128 },
}

fn cell_count(cells: u128) -> Result<usize, LayoutError> {
    usize::try_from(cells).map_err(|_| LayoutError::CellCountOverflow { cells })
}

fn triangle_side(axis_len: usize, other_len: usize, offset: usize) -> Result<usize, LayoutError> {
    let side = axis_len
        .checked_sub(offset)
        .ok_or(LayoutError::OffsetOutOfBounds { offset, axis_len })?;
    if side > other_len {
        return Err(LayoutError::TriangleDoesNotFit {
            side,
            available: other_len,
        });
    }
    Ok(side)
}

#[inline]
fn triangle_factors(side: usize) -> (usize, usize) {
    // Halve the even factor of side * (side + 1) before multiplying.
    (side.div_ceil(2), side | 1)
}

fn checked_triangle_cells(side: usize) -> Result<usize, LayoutError> {
    let (a, b) = triangle_factors(side);
    a.checked_mul(b)
        .ok_or_else(|| LayoutError::CellCountOverflow {
            cells: a as u128 * b as u128,
        })
}

/// Counts a triangle bounded by the side already validated at construction.
#[inline]
fn triangle_cells(side: usize) -> usize {
    let (a, b) = triangle_factors(side);
    a * b
}

/// The cells to visit in a scan. Column indices refer to sequence 2 and row indices to sequence 1.
///
/// # Safety
///
/// All methods must describe the same region throughout a scan.
///
/// - Every column in `columns()` visits a nonempty row prefix starting at zero.
///   All other columns have empty row ranges.
/// - Each visited row belongs to a single contiguous interval of columns.
///   A row can enter the scan and later leave, but cannot reappear after leaving.
///   The kernel relies on this to initialize rolling scores once per scan.
/// - `max_rows()` is the largest column height, or zero for an empty region.
/// - `len()` is the total cell count. Valid coordinates map to consecutive
///   indices in `0..len()` in column-major order.
pub unsafe trait Layout {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Absolute indices of the columns to visit.
    fn columns(&self) -> Range<usize>;

    /// Rows to visit in this column. Returns an empty range outside `columns()`.
    fn rows(&self, col: usize) -> Range<usize>;

    /// Exclusive upper bound of visited row indices, or zero for an empty layout.
    fn max_rows(&self) -> usize;

    /// Returns the dense index of a cell in the layout.
    ///
    /// # Safety
    /// The caller must ensure that `self.rows(col).contains(&row)`.
    unsafe fn index(&self, col: usize, row: usize) -> usize;

    /// Returns the index of a cell in the layout, or `None` outside the region.
    #[inline]
    fn get_index(&self, col: usize, row: usize) -> Option<usize> {
        if !self.rows(col).contains(&row) {
            return None;
        }
        // SAFETY: The row is in the range advertised for this column.
        Some(unsafe { self.index(col, row) })
    }
}

/// The entire rectangle spanned by the two sequences.
#[derive(Debug)]
pub struct FullMatrix {
    rows: usize,
    cols: usize,
    cells: usize,
}

impl FullMatrix {
    /// Returns [`LayoutError::CellCountOverflow`] if the rectangle has more than `usize::MAX` cells.
    pub fn new(seq1_len: usize, seq2_len: usize) -> Result<Self, LayoutError> {
        Ok(Self {
            rows: seq1_len,
            cols: if seq1_len == 0 { 0 } else { seq2_len },
            cells: cell_count(seq1_len as u128 * seq2_len as u128)?,
        })
    }
}

// SAFETY: The row prefix is constant and the constructor checks the full cell count.
unsafe impl Layout for FullMatrix {
    #[inline]
    fn len(&self) -> usize {
        self.cells
    }

    fn columns(&self) -> Range<usize> {
        0..self.cols
    }

    fn max_rows(&self) -> usize {
        if self.cols == 0 { 0 } else { self.rows }
    }

    fn rows(&self, col: usize) -> Range<usize> {
        if col < self.cols { 0..self.rows } else { 0..0 }
    }

    #[inline]
    unsafe fn index(&self, col: usize, row: usize) -> usize {
        debug_assert!(self.rows(col).contains(&row));
        col * self.rows + row
    }
}

/// Triangle bounded by the anti-diagonal `row + col + offset < seq1_len`.
/// An offset of zero includes the anti-diagonal through the last row of the first column.
/// Its side is `seq1_len - offset` and must fit in the columns of sequence 2.
///
/// For two sequences of length six, each `x` marks a cell to visit.
///
/// ```text
/// offset = 0                 offset = 2
///       0 1 2 3 4 5                0 1 2 3 4 5
///     0 x x x x x x              0 x x x x . .
///     1 x x x x x .              1 x x x . . .
///     2 x x x x . .              2 x x . . . .
///     3 x x x . . .              3 x . . . . .
///     4 x x . . . .              4 . . . . . .
///     5 x . . . . .              5 . . . . . .
/// ```
#[derive(Debug)]
pub struct AboveAntiDiagonal {
    side: usize,
    cells: usize,
}

impl AboveAntiDiagonal {
    /// Builds the entire triangle. Requires `offset <= seq1_len`, `seq1_len - offset <= seq2_len`,
    /// and a cell count representable as `usize`.
    pub fn new(seq1_len: usize, seq2_len: usize, offset: usize) -> Result<Self, LayoutError> {
        let side = triangle_side(seq1_len, seq2_len, offset)?;
        let cells = checked_triangle_cells(side)?;
        Ok(Self { side, cells })
    }
}

// SAFETY: Each row prefix shrinks by one per column. The constructor checks the
// cell count, and subtracting the remaining triangle gives each column's dense offset.
unsafe impl Layout for AboveAntiDiagonal {
    #[inline]
    fn len(&self) -> usize {
        self.cells
    }

    fn columns(&self) -> Range<usize> {
        0..self.side
    }

    fn max_rows(&self) -> usize {
        self.side
    }

    fn rows(&self, col: usize) -> Range<usize> {
        if col < self.side {
            0..self.side - col
        } else {
            0..0
        }
    }

    #[inline]
    unsafe fn index(&self, col: usize, row: usize) -> usize {
        debug_assert!(self.rows(col).contains(&row));
        // The remaining triangle fits in the cell count checked at construction.
        self.cells - triangle_cells(self.side - col) + row
    }
}

/// Upper triangle on or above the main diagonal shifted by `offset` columns.
/// A cell belongs to the triangle when `row + offset <= col`. The side `seq2_len - offset` must fit in sequence 1.
/// An offset of zero includes the main diagonal. An offset of one excludes it.
/// For a 3-by-5 matrix with offset two, columns 2, 3, and 4 visit one, two, and three rows.
#[derive(Debug)]
pub struct AboveMainDiagonal {
    cols: Range<usize>,
    cells: usize,
}

impl AboveMainDiagonal {
    /// Builds the entire triangle. Requires `offset <= seq2_len`, `seq2_len - offset <= seq1_len`,
    /// and a cell count representable as `usize`.
    pub fn new(seq1_len: usize, seq2_len: usize, offset: usize) -> Result<Self, LayoutError> {
        let side = triangle_side(seq2_len, seq1_len, offset)?;
        let cells = checked_triangle_cells(side)?;
        let cols = if side == 0 { 0..0 } else { offset..seq2_len };
        Ok(Self { cols, cells })
    }
}

// SAFETY: Each row prefix grows by one per column. The constructor checks the
// cell count, and the preceding triangle gives each column's dense offset.
unsafe impl Layout for AboveMainDiagonal {
    fn len(&self) -> usize {
        self.cells
    }

    #[inline]
    fn columns(&self) -> Range<usize> {
        self.cols.clone()
    }

    fn max_rows(&self) -> usize {
        self.cols.len()
    }

    #[inline]
    fn rows(&self, col: usize) -> Range<usize> {
        if self.cols.contains(&col) {
            0..col - self.cols.start + 1
        } else {
            0..0
        }
    }

    #[inline]
    unsafe fn index(&self, col: usize, row: usize) -> usize {
        debug_assert!(self.rows(col).contains(&row));
        triangle_cells(col - self.cols.start) + row
    }
}
