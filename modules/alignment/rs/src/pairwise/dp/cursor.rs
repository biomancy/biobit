/// A position in a column-major scan.
///
/// The caller is responsible for keeping the coordinates and dense index
/// consistent with the layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
    pub index: usize,
}

impl Cursor {
    /// Starts at the first cell of a scan.
    pub const fn new(first_col: usize) -> Self {
        Self {
            col: first_col,
            row: 0,
            index: 0,
        }
    }

    /// Advances by `count` rows within a column.
    #[inline]
    pub fn next_rows(&mut self, count: usize) {
        self.row += count;
        self.index += count;
    }

    /// Starts the next column after the caller has advanced past the current column's rows.
    #[inline]
    pub fn next_col(&mut self) {
        self.col += 1;
        self.row = 0;
    }
}
