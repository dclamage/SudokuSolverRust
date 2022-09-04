//! Contains [`CellIndex`] for representing the location of a specific cell.

use crate::prelude::*;

/// Represents the location of a specific cell on a board.
///
/// A 9x9 grid has 81 cells and are indexed 0 to 80.
///
/// The CellIndex is not linked to a specific board, but it
/// is linked to the size of the board.
/// A 9x9 grid has a size of 9.
#[derive(Clone, Copy, Debug)]
pub struct CellIndex {
    index: usize,
    size: usize,
}

impl CellIndex {
    /// Creates a new instance from a cell index.
    pub fn new(index: usize, size: usize) -> Self {
        Self { index, size }
    }

    /// Creates a new instance from a row and column index.
    pub fn from_rc(row: usize, column: usize, size: usize) -> Self {
        Self {
            index: row * size + column,
            size,
        }
    }

    /// Gets the index of the cell.
    pub fn index(self) -> usize {
        self.index
    }

    /// Gets the size of the grid being used for calculations.
    pub fn size(self) -> usize {
        self.size
    }

    /// Gets the row of the cell.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let cell = CellIndex::new(0, 9);
    /// assert_eq!(cell.row(), 0);
    ///
    /// let cell = CellIndex::new(8, 9);
    /// assert_eq!(cell.row(), 0);
    ///
    /// let cell = CellIndex::new(9, 9);
    /// assert_eq!(cell.row(), 1);
    ///
    /// let cell = CellIndex::new(80, 9);
    /// assert_eq!(cell.row(), 8);
    /// ```
    pub fn row(self) -> usize {
        self.index / self.size
    }

    /// Gets the column of the cell.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let cell = CellIndex::new(0, 9);
    /// assert_eq!(cell.column(), 0);
    ///
    /// let cell = CellIndex::new(8, 9);
    /// assert_eq!(cell.column(), 8);
    ///
    /// let cell = CellIndex::new(9, 9);
    /// assert_eq!(cell.column(), 0);
    ///
    /// let cell = CellIndex::new(80, 9);
    /// assert_eq!(cell.column(), 8);
    ///
    /// let cell = CellIndex::new(81, 9);
    /// assert_eq!(cell.column(), 0);
    /// ```
    pub fn column(self) -> usize {
        self.index % self.size
    }

    /// Gets the row and column of the cell.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let cell = CellIndex::new(0, 9);
    /// assert_eq!(cell.rc(), (0, 0));
    ///
    /// let cell = CellIndex::new(8, 9);
    /// assert_eq!(cell.rc(), (0, 8));
    ///
    /// let cell = CellIndex::new(9, 9);
    /// assert_eq!(cell.rc(), (1, 0));
    ///
    /// let cell = CellIndex::new(80, 9);
    /// assert_eq!(cell.rc(), (8, 8));
    ///
    /// let cell = CellIndex::new(81, 9);
    /// assert_eq!(cell.rc(), (9, 0));
    /// ```
    pub fn rc(self) -> (usize, usize) {
        (self.row(), self.column())
    }

    /// Gets the [`CandidateIndex`] of a value in this cell
    pub fn candidate(self, value: usize) -> CandidateIndex {
        CandidateIndex::from_cv(self, value)
    }

    /// Gets the [`CandidateIndex`] of all values in this cell
    pub fn all_candidates(self) -> Vec<CandidateIndex> {
        (0..self.size).map(|value| self.candidate(value)).collect()
    }

    /// Gets the taxicab distance between two cells.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let cell1 = CellIndex::new(0, 9);
    /// let cell2 = CellIndex::new(0, 9);
    /// assert_eq!(cell1.taxicab_distance(cell2), 0);
    ///
    /// let cell1 = CellIndex::new(0, 9);
    /// let cell2 = CellIndex::new(1, 9);
    /// assert_eq!(cell1.taxicab_distance(cell2), 1);
    ///
    /// let cell1 = CellIndex::new(0, 9);
    /// let cell2 = CellIndex::new(9, 9);
    /// assert_eq!(cell1.taxicab_distance(cell2), 1);
    ///
    /// let cell1 = CellIndex::new(0, 9);
    /// let cell2 = CellIndex::new(10, 9);
    /// assert_eq!(cell1.taxicab_distance(cell2), 2);
    ///
    /// let cell1 = CellIndex::new(0, 9);
    /// let cell2 = CellIndex::new(80, 9);
    /// assert_eq!(cell1.taxicab_distance(cell2), 16);
    /// ```
    pub fn taxicab_distance(self, other: Self) -> usize {
        let (row1, column1) = self.rc();
        let (row2, column2) = other.rc();
        (row1 as isize - row2 as isize).unsigned_abs()
            + (column1 as isize - column2 as isize).unsigned_abs()
    }

    /// Determines if two cells are orthogonally adjacent.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(0, 0, 9);
    /// assert!(!cell1.is_orthogonally_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(0, 1, 9);
    /// assert!(cell1.is_orthogonally_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(1, 0, 9);
    /// assert!(cell1.is_orthogonally_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(1, 1, 9);
    /// assert!(!cell1.is_orthogonally_adjacent(cell2));
    /// ```
    pub fn is_orthogonally_adjacent(self, other: Self) -> bool {
        let (row1, column1) = self.rc();
        let (row2, column2) = other.rc();
        row1 == row2 && (column1 as isize - column2 as isize).abs() == 1
            || column1 == column2 && (row1 as isize - row2 as isize).abs() == 1
    }

    /// Determines if two cells are diagonally adjacent.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(0, 0, 9);
    /// assert!(!cell1.is_diagonally_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(0, 1, 9);
    /// assert!(!cell1.is_diagonally_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(1, 0, 9);
    /// assert!(!cell1.is_diagonally_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(1, 1, 9);
    /// assert!(cell1.is_diagonally_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(1, 2, 9);
    /// assert!(!cell1.is_diagonally_adjacent(cell2));
    /// ```
    pub fn is_diagonally_adjacent(self, other: Self) -> bool {
        let (row1, column1) = self.rc();
        let (row2, column2) = other.rc();
        (row1 as isize - row2 as isize).abs() == 1
            && (column1 as isize - column2 as isize).abs() == 1
    }

    /// Determines if two cells are adjacent.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(0, 0, 9);
    /// assert!(!cell1.is_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(0, 1, 9);
    /// assert!(cell1.is_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(1, 0, 9);
    /// assert!(cell1.is_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(1, 1, 9);
    /// assert!(cell1.is_adjacent(cell2));
    ///
    /// let cell1 = CellIndex::from_rc(0, 0, 9);
    /// let cell2 = CellIndex::from_rc(1, 2, 9);
    /// assert!(!cell1.is_adjacent(cell2));
    /// ```
    pub fn is_adjacent(self, other: Self) -> bool {
        self.is_orthogonally_adjacent(other) || self.is_diagonally_adjacent(other)
    }

    /// Returns a vector of all cells that are orthogonally adjacent to this cell.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let cell = CellIndex::from_rc(0, 0, 9);
    /// let adjacent_cells = cell.orthogonally_adjacent_cells();
    /// assert_eq!(adjacent_cells.len(), 2);
    /// assert_eq!(adjacent_cells, vec![CellIndex::from_rc(0, 1, 9), CellIndex::from_rc(1, 0, 9)]);
    ///
    /// let cell = CellIndex::from_rc(0, 1, 9);
    /// let adjacent_cells = cell.orthogonally_adjacent_cells();
    /// assert_eq!(adjacent_cells.len(), 3);
    /// assert_eq!(adjacent_cells, vec![CellIndex::from_rc(0, 0, 9), CellIndex::from_rc(0, 2, 9), CellIndex::from_rc(1, 1, 9)]);
    ///
    /// let cell = CellIndex::from_rc(1, 1, 9);
    /// let adjacent_cells = cell.orthogonally_adjacent_cells();
    /// assert_eq!(adjacent_cells.len(), 4);
    /// assert_eq!(adjacent_cells, vec![CellIndex::from_rc(0, 1, 9), CellIndex::from_rc(1, 0, 9), CellIndex::from_rc(1, 2, 9), CellIndex::from_rc(2, 1, 9)]);
    /// ```
    pub fn orthogonally_adjacent_cells(self) -> Vec<Self> {
        let (row, column) = self.rc();
        let mut adjacent_cells = Vec::new();
        if row > 0 {
            adjacent_cells.push(Self::from_rc(row - 1, column, self.size));
        }
        if row < self.size - 1 {
            adjacent_cells.push(Self::from_rc(row + 1, column, self.size));
        }
        if column > 0 {
            adjacent_cells.push(Self::from_rc(row, column - 1, self.size));
        }
        if column < self.size - 1 {
            adjacent_cells.push(Self::from_rc(row, column + 1, self.size));
        }
        adjacent_cells.sort();
        adjacent_cells
    }

    /// Returns a vector of all cells that are diagonally adjacent to this cell.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let cell = CellIndex::from_rc(0, 0, 9);
    /// let adjacent_cells = cell.diagonally_adjacent_cells();
    /// assert_eq!(adjacent_cells.len(), 1);
    /// assert_eq!(adjacent_cells, vec![CellIndex::from_rc(1, 1, 9)]);
    ///
    /// let cell = CellIndex::from_rc(0, 1, 9);
    /// let adjacent_cells = cell.diagonally_adjacent_cells();
    /// assert_eq!(adjacent_cells.len(), 2);
    /// assert_eq!(adjacent_cells, vec![CellIndex::from_rc(1, 0, 9), CellIndex::from_rc(1, 2, 9)]);
    ///
    /// let cell = CellIndex::from_rc(1, 1, 9);
    /// let adjacent_cells = cell.diagonally_adjacent_cells();
    /// assert_eq!(adjacent_cells.len(), 4);
    /// assert_eq!(adjacent_cells, vec![CellIndex::from_rc(0, 0, 9), CellIndex::from_rc(0, 2, 9), CellIndex::from_rc(2, 0, 9), CellIndex::from_rc(2, 2, 9)]);
    /// ```
    pub fn diagonally_adjacent_cells(self) -> Vec<Self> {
        let (row, column) = self.rc();
        let mut adjacent_cells = Vec::new();
        if row > 0 && column > 0 {
            adjacent_cells.push(Self::from_rc(row - 1, column - 1, self.size));
        }
        if row > 0 && column < self.size - 1 {
            adjacent_cells.push(Self::from_rc(row - 1, column + 1, self.size));
        }
        if row < self.size - 1 && column > 0 {
            adjacent_cells.push(Self::from_rc(row + 1, column - 1, self.size));
        }
        if row < self.size - 1 && column < self.size - 1 {
            adjacent_cells.push(Self::from_rc(row + 1, column + 1, self.size));
        }
        adjacent_cells.sort();
        adjacent_cells
    }

    /// Returns a vector of all cells that are adjacent to this cell.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let cell = CellIndex::from_rc(0, 0, 9);
    /// let adjacent_cells = cell.adjacent_cells();
    /// assert_eq!(adjacent_cells.len(), 3);
    /// assert_eq!(adjacent_cells, vec![CellIndex::from_rc(0, 1, 9), CellIndex::from_rc(1, 0, 9), CellIndex::from_rc(1, 1, 9)]);
    ///
    /// let cell = CellIndex::from_rc(0, 1, 9);
    /// let adjacent_cells = cell.adjacent_cells();
    /// assert_eq!(adjacent_cells.len(), 5);
    /// assert_eq!(adjacent_cells, vec![CellIndex::from_rc(0, 0, 9), CellIndex::from_rc(0, 2, 9), CellIndex::from_rc(1, 0, 9), CellIndex::from_rc(1, 1, 9), CellIndex::from_rc(1, 2, 9)]);
    ///
    /// let cell = CellIndex::from_rc(1, 1, 9);
    /// let adjacent_cells = cell.adjacent_cells();
    /// assert_eq!(adjacent_cells.len(), 8);
    /// assert_eq!(adjacent_cells, vec![CellIndex::from_rc(0, 0, 9), CellIndex::from_rc(0, 1, 9), CellIndex::from_rc(0, 2, 9), CellIndex::from_rc(1, 0, 9), CellIndex::from_rc(1, 2, 9), CellIndex::from_rc(2, 0, 9), CellIndex::from_rc(2, 1, 9), CellIndex::from_rc(2, 2, 9)]);
    /// ```
    pub fn adjacent_cells(self) -> Vec<Self> {
        let (row, column) = self.rc();
        let mut adjacent_cells = Vec::new();
        if row > 0 {
            adjacent_cells.push(Self::from_rc(row - 1, column, self.size));
        }
        if row < self.size - 1 {
            adjacent_cells.push(Self::from_rc(row + 1, column, self.size));
        }
        if column > 0 {
            adjacent_cells.push(Self::from_rc(row, column - 1, self.size));
        }
        if column < self.size - 1 {
            adjacent_cells.push(Self::from_rc(row, column + 1, self.size));
        }
        if row > 0 && column > 0 {
            adjacent_cells.push(Self::from_rc(row - 1, column - 1, self.size));
        }
        if row > 0 && column < self.size - 1 {
            adjacent_cells.push(Self::from_rc(row - 1, column + 1, self.size));
        }
        if row < self.size - 1 && column > 0 {
            adjacent_cells.push(Self::from_rc(row + 1, column - 1, self.size));
        }
        if row < self.size - 1 && column < self.size - 1 {
            adjacent_cells.push(Self::from_rc(row + 1, column + 1, self.size));
        }
        adjacent_cells.sort();
        adjacent_cells
    }
}

impl std::fmt::Display for CellIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let (row, column) = self.rc();
        write!(f, "r{}c{}", row + 1, column + 1)
    }
}

impl Eq for CellIndex {}

impl PartialEq for CellIndex {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Ord for CellIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl PartialOrd for CellIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for CellIndex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cell_index() {
        assert_eq!(CellIndex::from_rc(0, 0, 9).index(), 0);
        assert_eq!(CellIndex::from_rc(1, 0, 9).index(), 9);
        assert_eq!(CellIndex::from_rc(1, 0, 16).index(), 16);
        assert_eq!(CellIndex::from_rc(1, 1, 8).index(), 9);
        assert_eq!(CellIndex::from_rc(1, 1, 16).index(), 17);
        assert_eq!(CellIndex::from_rc(1, 2, 16).index(), 18);
        assert_eq!(CellIndex::from_rc(8, 8, 9).index(), 80);
        assert_eq!(CellIndex::from_rc(4, 4, 9).index(), 40);
    }

    #[test]
    fn test_cell_row_col() {
        assert_eq!(CellIndex::new(0, 9).rc(), (0, 0));
        assert_eq!(CellIndex::new(1, 9).rc(), (0, 1));
        assert_eq!(CellIndex::new(16, 16).rc(), (1, 0));
        assert_eq!(CellIndex::new(9, 8).rc(), (1, 1));
        assert_eq!(CellIndex::new(17, 16).rc(), (1, 1));
        assert_eq!(CellIndex::new(18, 16).rc(), (1, 2));
        assert_eq!(CellIndex::new(80, 9).rc(), (8, 8));
        assert_eq!(CellIndex::new(40, 9).rc(), (4, 4));
    }

    #[test]
    fn test_orthogonally_adjacent_cells() {
        let cu = crate::cell_utility::CellUtility::new(9);
        assert_eq!(
            cu.cell(0, 0).orthogonally_adjacent_cells(),
            vec![cu.cell(0, 1), cu.cell(1, 0)]
        );
        assert_eq!(
            cu.cell(0, 1).orthogonally_adjacent_cells(),
            vec![cu.cell(0, 0), cu.cell(0, 2), cu.cell(1, 1)]
        );
        assert_eq!(
            cu.cell(0, 2).orthogonally_adjacent_cells(),
            vec![cu.cell(0, 1), cu.cell(0, 3), cu.cell(1, 2)]
        );
        assert_eq!(
            cu.cell(0, 3).orthogonally_adjacent_cells(),
            vec![cu.cell(0, 2), cu.cell(0, 4), cu.cell(1, 3)]
        );
        assert_eq!(
            cu.cell(0, 4).orthogonally_adjacent_cells(),
            vec![cu.cell(0, 3), cu.cell(0, 5), cu.cell(1, 4)]
        );
        assert_eq!(
            cu.cell(0, 5).orthogonally_adjacent_cells(),
            vec![cu.cell(0, 4), cu.cell(0, 6), cu.cell(1, 5)]
        );
        assert_eq!(
            cu.cell(0, 6).orthogonally_adjacent_cells(),
            vec![cu.cell(0, 5), cu.cell(0, 7), cu.cell(1, 6)]
        );
        assert_eq!(
            cu.cell(0, 7).orthogonally_adjacent_cells(),
            vec![cu.cell(0, 6), cu.cell(0, 8), cu.cell(1, 7)]
        );
        assert_eq!(
            cu.cell(0, 8).orthogonally_adjacent_cells(),
            vec![cu.cell(0, 7), cu.cell(1, 8)]
        );
        assert_eq!(
            cu.cell(1, 0).orthogonally_adjacent_cells(),
            vec![cu.cell(0, 0), cu.cell(1, 1), cu.cell(2, 0)]
        );
        assert_eq!(
            cu.cell(4, 4).orthogonally_adjacent_cells(),
            vec![cu.cell(3, 4), cu.cell(4, 3), cu.cell(4, 5), cu.cell(5, 4)]
        );
        assert_eq!(
            cu.cell(8, 8).orthogonally_adjacent_cells(),
            vec![cu.cell(7, 8), cu.cell(8, 7)]
        );
    }

    #[test]
    fn test_diagonal_cells() {
        let cu = crate::cell_utility::CellUtility::new(9);
        assert_eq!(
            cu.cell(0, 0).diagonally_adjacent_cells(),
            vec![cu.cell(1, 1)]
        );
        assert_eq!(
            cu.cell(0, 1).diagonally_adjacent_cells(),
            vec![cu.cell(1, 0), cu.cell(1, 2)]
        );
        assert_eq!(
            cu.cell(0, 2).diagonally_adjacent_cells(),
            vec![cu.cell(1, 1), cu.cell(1, 3)]
        );
        assert_eq!(
            cu.cell(0, 3).diagonally_adjacent_cells(),
            vec![cu.cell(1, 2), cu.cell(1, 4)]
        );
        assert_eq!(
            cu.cell(0, 4).diagonally_adjacent_cells(),
            vec![cu.cell(1, 3), cu.cell(1, 5)]
        );
        assert_eq!(
            cu.cell(0, 5).diagonally_adjacent_cells(),
            vec![cu.cell(1, 4), cu.cell(1, 6)]
        );
        assert_eq!(
            cu.cell(0, 6).diagonally_adjacent_cells(),
            vec![cu.cell(1, 5), cu.cell(1, 7)]
        );
        assert_eq!(
            cu.cell(0, 7).diagonally_adjacent_cells(),
            vec![cu.cell(1, 6), cu.cell(1, 8)]
        );
        assert_eq!(
            cu.cell(0, 8).diagonally_adjacent_cells(),
            vec![cu.cell(1, 7)]
        );
        assert_eq!(
            cu.cell(1, 0).diagonally_adjacent_cells(),
            vec![cu.cell(0, 1), cu.cell(2, 1)]
        );
        assert_eq!(
            cu.cell(4, 4).diagonally_adjacent_cells(),
            vec![cu.cell(3, 3), cu.cell(3, 5), cu.cell(5, 3), cu.cell(5, 5)]
        );
        assert_eq!(
            cu.cell(8, 8).diagonally_adjacent_cells(),
            vec![cu.cell(7, 7)]
        );
    }

    #[test]
    fn test_adjacent_cells() {
        let cu = crate::cell_utility::CellUtility::new(9);
        assert_eq!(
            cu.cell(0, 0).adjacent_cells(),
            vec![cu.cell(0, 1), cu.cell(1, 0), cu.cell(1, 1),]
        );
        assert_eq!(
            cu.cell(0, 1).adjacent_cells(),
            vec![
                cu.cell(0, 0),
                cu.cell(0, 2),
                cu.cell(1, 0),
                cu.cell(1, 1),
                cu.cell(1, 2),
            ]
        );
        assert_eq!(
            cu.cell(0, 2).adjacent_cells(),
            vec![
                cu.cell(0, 1),
                cu.cell(0, 3),
                cu.cell(1, 1),
                cu.cell(1, 2),
                cu.cell(1, 3),
            ]
        );
        assert_eq!(
            cu.cell(0, 3).adjacent_cells(),
            vec![
                cu.cell(0, 2),
                cu.cell(0, 4),
                cu.cell(1, 2),
                cu.cell(1, 3),
                cu.cell(1, 4),
            ]
        );
        assert_eq!(
            cu.cell(0, 4).adjacent_cells(),
            vec![
                cu.cell(0, 3),
                cu.cell(0, 5),
                cu.cell(1, 3),
                cu.cell(1, 4),
                cu.cell(1, 5),
            ]
        );
        assert_eq!(
            cu.cell(0, 5).adjacent_cells(),
            vec![
                cu.cell(0, 4),
                cu.cell(0, 6),
                cu.cell(1, 4),
                cu.cell(1, 5),
                cu.cell(1, 6),
            ]
        );
        assert_eq!(
            cu.cell(0, 6).adjacent_cells(),
            vec![
                cu.cell(0, 5),
                cu.cell(0, 7),
                cu.cell(1, 5),
                cu.cell(1, 6),
                cu.cell(1, 7),
            ]
        );
        assert_eq!(
            cu.cell(0, 7).adjacent_cells(),
            vec![
                cu.cell(0, 6),
                cu.cell(0, 8),
                cu.cell(1, 6),
                cu.cell(1, 7),
                cu.cell(1, 8),
            ]
        );
        assert_eq!(
            cu.cell(0, 8).adjacent_cells(),
            vec![cu.cell(0, 7), cu.cell(1, 7), cu.cell(1, 8),]
        );
        assert_eq!(
            cu.cell(1, 0).adjacent_cells(),
            vec![
                cu.cell(0, 0),
                cu.cell(0, 1),
                cu.cell(1, 1),
                cu.cell(2, 0),
                cu.cell(2, 1),
            ]
        );
        assert_eq!(
            cu.cell(4, 4).adjacent_cells(),
            vec![
                cu.cell(3, 3),
                cu.cell(3, 4),
                cu.cell(3, 5),
                cu.cell(4, 3),
                cu.cell(4, 5),
                cu.cell(5, 3),
                cu.cell(5, 4),
                cu.cell(5, 5),
            ]
        );
        assert_eq!(
            cu.cell(8, 8).adjacent_cells(),
            vec![cu.cell(7, 7), cu.cell(7, 8), cu.cell(8, 7),]
        );
    }
}
