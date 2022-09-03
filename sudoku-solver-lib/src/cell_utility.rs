//! Contains [`CellUtility`] which has methods for working with cells.

use crate::{candidate_index::CandidateIndex, cell_index::CellIndex};
use itertools::Itertools;

/// A utility struct for working with cells.
///
/// Use the [`CellUtility::new`] function to create a new instance with a specific
/// board size.
///
/// Since many of the functions in this struct require the board size,
/// by storing the size in the struct, we can avoid passing it as a
/// parameter to each function.
#[derive(Copy, Clone, Debug)]
pub struct CellUtility {
    size: usize,
}

impl CellUtility {
    /// Creates a new instance.
    pub fn new(size: usize) -> Self {
        Self { size }
    }

    /// Gets the size of the board.
    pub fn size(self) -> usize {
        self.size
    }

    /// Gets the number of cells in the board.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    /// assert_eq!(cu.cell_count(), 81);
    /// ```
    pub fn cell_count(self) -> usize {
        self.size * self.size
    }

    /// Gets the number of candidates in the board.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    /// assert_eq!(cu.candidate_count(), 729);
    /// ```
    pub fn candidate_count(self) -> usize {
        self.cell_count() * self.size
    }

    /// Creates a [`CellIndex`] from a row and column index.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    /// assert_eq!(cu.cell(0, 0), CellIndex::from_rc(0, 0, size));
    /// assert_eq!(cu.cell(1, 2), CellIndex::from_rc(1, 2, size));
    /// assert_eq!(cu.cell(8, 8), CellIndex::from_rc(8, 8, size));
    /// ```
    pub fn cell(self, row: usize, col: usize) -> CellIndex {
        CellIndex::new(row * self.size + col, self.size)
    }

    /// Creates a [`CellIndex`] from a linear index.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    /// assert_eq!(cu.cell_index(0), CellIndex::new(0, size));
    /// assert_eq!(cu.cell_index(11), CellIndex::new(11, size));
    /// assert_eq!(cu.cell_index(80), CellIndex::new(80, size));
    /// ```
    pub fn cell_index(self, index: usize) -> CellIndex {
        CellIndex::new(index, self.size)
    }

    /// Creates a [`CandidateIndex`] from a cell index and value.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// # use sudoku_solver_lib::candidate_index::CandidateIndex;
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    /// assert_eq!(cu.candidate(cu.cell(0, 0), 1), CandidateIndex::new(0, size));
    /// assert_eq!(cu.candidate(cu.cell(4, 4), 5), CandidateIndex::new(364, size));
    /// assert_eq!(cu.candidate(cu.cell(8, 8), 9), CandidateIndex::new(728, size));
    pub fn candidate(self, cell: CellIndex, value: usize) -> CandidateIndex {
        CandidateIndex::from_cv(cell, value)
    }

    /// Creates a [`CandidateIndex`] from a linear index.
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// # use sudoku_solver_lib::candidate_index::CandidateIndex;
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    /// assert_eq!(cu.candidate_index(0), CandidateIndex::new(0, size));
    /// assert_eq!(cu.candidate_index(11), CandidateIndex::new(11, size));
    /// assert_eq!(cu.candidate_index(728), CandidateIndex::new(728, size));
    /// ```
    pub fn candidate_index(self, index: usize) -> CandidateIndex {
        CandidateIndex::new(index, self.size)
    }

    /// Creates an iterator over all cells in the board.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    /// let cells: Vec<CellIndex> = cu.all_cells().collect();
    /// assert_eq!(cells.len(), 81);
    /// assert_eq!(cells[0], CellIndex::new(0, size));
    /// assert_eq!(cells[80], CellIndex::new(80, size));
    /// ```
    pub fn all_cells(self) -> impl Iterator<Item = CellIndex> {
        (0..self.cell_count()).map(move |i| self.cell_index(i))
    }

    /// Creates an iterator over all candidates in the board.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// # use sudoku_solver_lib::candidate_index::CandidateIndex;
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    /// let candidates: Vec<CandidateIndex> = cu.all_candidates().collect();
    /// assert_eq!(candidates.len(), 729);
    /// assert_eq!(candidates[0], CandidateIndex::new(0, size));
    /// assert_eq!(candidates[728], CandidateIndex::new(728, size));
    /// ```
    pub fn all_candidates(self) -> impl Iterator<Item = CandidateIndex> {
        (0..self.candidate_count()).map(move |i| self.candidate_index(i))
    }

    /// Parses a string into a list of groups of cells.
    ///
    /// The string is expected to be a sequence of groups of cells, separated by
    /// semi-colons. Each group is a set of cells.
    ///
    /// Groups can be specified in a multitude of ways. Groups are specified as one
    /// or more sub-groups with nothing in between. Styles of specified sub-groups
    /// can be mixed together in a single group.
    ///
    /// Sub-groups can be specified as a single cell, a list of cells, a range of cells,
    /// a disjoint range of cells, or a starting cell with numpad directions.
    /// Sub-group Examples:
    /// * r2c3 - a single cell at row 2, column 3
    /// * r2c3r4c5 - two cells, one at row 2, column 3, and one at row 4, column 5
    /// * r1-4c5 - a range of 4 cells, from row 1 to row 4, all in column 5
    /// * r2c3-8 - a range of 6 cells, from column 3 to column 8, all in row 2
    /// * r2-4c3-6 - a range of 12 cells, from row 2 to row 4, and from column 3 to column 6
    /// * r1,3,5c5 - a disjoint range of 3 cells, row 1, row 3, and row 5, all in column 5
    /// * r2,4,6c1,3,5,7 - a disjoint range of 12 cells, row 2, row 4, and row 6, and column 1, column 3, column 5, and column 7
    /// * r1c1d2229 - a starting cell at row 1, column 1, and then adding cells as we go, moving down 3 times and up-right once.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    /// let groups = cu.parse_cell_groups("r2c3;r3c4r4c5").unwrap();
    /// assert_eq!(groups.len(), 2);
    /// assert_eq!(groups[0], vec![cu.cell(1, 2)]);
    /// assert_eq!(groups[1], vec![cu.cell(2, 3), cu.cell(3, 4)]);
    /// ```
    pub fn parse_cell_groups(self, cell_string: &str) -> Result<Vec<Vec<CellIndex>>, String> {
        let size = self.size;
        let mut result = Vec::new();

        for cell_group in cell_string
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            if !cell_group.is_ascii() {
                return Err(format!(
                    "Invalid cell group (invalid characters): {}",
                    cell_group
                ));
            }

            let err_msg = format!("Invalid cell group: {}", cell_group);
            let cell_group = cell_group.to_ascii_lowercase();
            let cell_group = cell_group.as_bytes();

            if cell_group.len() < 4 {
                return Err(err_msg);
            }

            let first_char = cell_group[0];
            if first_char != b'r' {
                return Err(err_msg);
            }

            let mut cells = Vec::new();

            let mut rows: Vec<usize> = Vec::new();
            let mut cols: Vec<usize> = Vec::new();
            let mut adding_rows = true;
            let mut value_start = true;
            let mut last_added_directions = false;
            let mut cur_val_start = 0;
            let mut cur_val_end = 0;
            let mut i = 1;
            while i < cell_group.len() {
                last_added_directions = false;

                let cur_char = cell_group[i];
                if cur_char == b'r' {
                    if adding_rows
                        || self
                            .add_range(&mut cols, cur_val_start, cur_val_end)
                            .is_err()
                        || self.add_cells(&mut cells, &rows, &cols).is_err()
                    {
                        return Err(err_msg);
                    }
                    rows.clear();
                    cols.clear();
                    adding_rows = true;
                    value_start = true;
                    cur_val_start = 0;
                    cur_val_end = 0;
                } else if cur_char == b'c' {
                    if !adding_rows
                        || self
                            .add_range(&mut rows, cur_val_start, cur_val_end)
                            .is_err()
                        || self.add_cells(&mut cells, &rows, &cols).is_err()
                    {
                        return Err(err_msg);
                    }
                    adding_rows = false;
                    value_start = true;
                    cur_val_start = 0;
                    cur_val_end = 0;
                } else if cur_char == b'd' {
                    if adding_rows
                        || self
                            .add_range(&mut cols, cur_val_start, cur_val_end)
                            .is_err()
                        || self.add_cells(&mut cells, &rows, &cols).is_err()
                    {
                        return Err(err_msg);
                    }
                    rows.clear();
                    cols.clear();
                    adding_rows = true;
                    value_start = true;
                    cur_val_start = 0;
                    cur_val_end = 0;

                    if cells.is_empty() {
                        return Err(err_msg);
                    }

                    i += 1;
                    let mut complete = false;
                    while i < cell_group.len() && !complete {
                        let cell = cells[cells.len() - 1];
                        let (r, c) = cell.rc();
                        let (r, c) = (r as isize, c as isize);
                        let dir_char = cell_group[i];
                        let mut to_add = (r, c);
                        match dir_char {
                            b'1' => to_add = (r + 1, c - 1),
                            b'2' => to_add = (r + 1, c),
                            b'3' => to_add = (r + 1, c + 1),
                            b'4' => to_add = (r, c - 1),
                            b'5' => to_add = (r, c),
                            b'6' => to_add = (r, c + 1),
                            b'7' => to_add = (r - 1, c - 1),
                            b'8' => to_add = (r - 1, c),
                            b'9' => to_add = (r - 1, c + 1),
                            b'r' => complete = true,
                            _ => {
                                return Err(err_msg);
                            }
                        }
                        if to_add.0 < 0
                            || to_add.0 > size as isize
                            || to_add.1 < 0
                            || to_add.1 > size as isize
                        {
                            return Err(err_msg);
                        }
                        cells.push(self.cell(to_add.0 as usize, to_add.1 as usize));
                        last_added_directions = true;
                        i += 1;
                    }
                    i -= 1;
                } else if (b'0'..=b'9').contains(&cur_char) {
                    if value_start {
                        cur_val_start = cur_val_start * 10 + (cur_char - b'0') as usize;
                    } else {
                        cur_val_end = cur_val_end * 10 + (cur_char - b'0') as usize;
                    }
                } else if cur_char == b'-' {
                    if !value_start {
                        return Err(err_msg);
                    }
                    value_start = false;
                } else if cur_char == b',' {
                    if self
                        .add_range(
                            if adding_rows { &mut rows } else { &mut cols },
                            cur_val_start,
                            cur_val_end,
                        )
                        .is_err()
                    {
                        return Err(err_msg);
                    }
                    value_start = true;
                    cur_val_start = 0;
                    cur_val_end = 0;
                } else {
                    return Err(err_msg);
                }

                i += 1;
            }

            if !last_added_directions
                && (adding_rows
                    || self
                        .add_range(&mut cols, cur_val_start, cur_val_end)
                        .is_err()
                    || self.add_cells(&mut cells, &rows, &cols).is_err()
                    || cells.is_empty())
            {
                return Err(err_msg);
            }

            result.push(cells);
        }

        result.sort();

        Result::Ok(result)
    }

    // Used by parse_cell_groups
    fn add_range(self, list: &mut Vec<usize>, start: usize, end: usize) -> Result<(), ()> {
        if start == 0 {
            return Err(());
        }

        if end == 0 {
            list.push(start);
        } else {
            let start = usize::min(start, end);
            let end = usize::max(start, end);
            for i in start..=end {
                list.push(i);
            }
        }

        Ok(())
    }

    // Used by parse_cell_groups
    fn add_cells(
        self,
        list: &mut Vec<CellIndex>,
        rows: &[usize],
        cols: &[usize],
    ) -> Result<(), ()> {
        let size = self.size;

        for &r in rows {
            for &c in cols {
                if r == 0 || c == 0 || r > size || c > size {
                    return Err(());
                }

                list.push(self.cell(r - 1, c - 1));
            }
        }

        Ok(())
    }

    /// Returns a vector of candidate pairs for each value within the given set of cells.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// # use sudoku_solver_lib::cell_index::CellIndex;
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    /// let cells = vec![cu.cell(0, 0), cu.cell(0, 1), cu.cell(0, 2)];
    /// let pairs = cu.candidate_pairs(&cells);
    /// assert_eq!(pairs.len(), 27);
    /// let cand1r1c1 = cu.candidate(cu.cell(0, 0), 1);
    /// let cand1r1c2 = cu.candidate(cu.cell(0, 1), 1);
    /// let cand1r1c3 = cu.candidate(cu.cell(0, 2), 1);
    /// assert!(pairs.contains(&(cand1r1c1, cand1r1c2)));
    /// assert!(pairs.contains(&(cand1r1c1, cand1r1c3)));
    /// assert!(pairs.contains(&(cand1r1c2, cand1r1c3)));
    ///
    /// let cand8r1c1 = cu.candidate(cu.cell(0, 0), 8);
    /// let cand8r1c2 = cu.candidate(cu.cell(0, 1), 8);
    /// let cand8r1c3 = cu.candidate(cu.cell(0, 2), 8);
    /// assert!(pairs.contains(&(cand8r1c1, cand8r1c2)));
    /// assert!(pairs.contains(&(cand8r1c1, cand8r1c3)));
    /// assert!(pairs.contains(&(cand8r1c2, cand8r1c3)));
    /// ```
    pub fn candidate_pairs(self, cells: &[CellIndex]) -> Vec<(CandidateIndex, CandidateIndex)> {
        let mut result = Vec::new();
        for val in 1..=self.size {
            for cell_pair in cells.iter().combinations(2) {
                let cand0 = self.candidate(*cell_pair[0], val);
                let cand1 = self.candidate(*cell_pair[1], val);
                result.push((cand0, cand1));
            }
        }
        result
    }

    /// Generates a compact description of a group of cells.
    ///
    /// # Returns
    /// A string describing the cells.
    /// - If they all share a row, then it returns for example `r1c123`
    /// - If they all share a column, then it returns for example `r123c1`
    /// - Otherwise, the cells are separated into groups like `r1c123,r2c123,r3c123`
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// // Assume a 9x9 grid.
    /// let size = 9;
    /// let cu = CellUtility::new(size);
    ///
    /// // Create a list of the following cells: r1c1, r1c2, r1c3
    /// // (Cell indices are 0-based)
    /// let cells = vec![cu.cell(0, 0), cu.cell(0, 1), cu.cell(0, 2)];
    ///
    /// // Get the compact name.
    /// let compact_name = cu.compact_name(&cells);
    /// assert_eq!(compact_name, "r1c123");
    /// ```
    pub fn compact_name(self, cells: &[CellIndex]) -> String {
        let size = self.size;
        let cell_separator = if size <= 9 { "" } else { "," };
        let group_separator = ",";

        if cells.is_empty() {
            return "".to_string();
        }

        if cells.len() == 1 {
            return cells[0].to_string();
        }

        let cells: Vec<(usize, usize)> = cells.iter().sorted().map(|cell| cell.rc()).collect();

        // If all share a row, group all by row
        let first_row = cells[0].0;
        if cells.iter().all(|cell| cell.0 == first_row) {
            return format!(
                "r{}c{}",
                cells[0].0 + 1,
                cells
                    .iter()
                    .map(|cell| cell.1 + 1)
                    .sorted()
                    .join(&cell_separator)
            );
        }

        // If all share a column, group all by column
        let first_col = cells[0].1;
        if cells.iter().all(|cell| cell.1 == first_col) {
            return format!(
                "r{}c{}",
                cells
                    .iter()
                    .map(|cell| cell.0 + 1)
                    .sorted()
                    .join(&cell_separator),
                cells[0].1 + 1
            );
        }

        // More complex case that spans rows and cols
        let grouped_by_row =
            self.compact_name_grouped_by_row(&cells, &cell_separator, &group_separator);
        let grouped_by_col =
            self.compact_name_grouped_by_col(&cells, &cell_separator, &group_separator);

        if grouped_by_row.len() < grouped_by_col.len() {
            grouped_by_row
        } else {
            grouped_by_col
        }
    }

    // Used by compact_name
    fn compact_name_grouped_by_row(
        self,
        cells: &[(usize, usize)],
        cell_separator: &str,
        group_separator: &str,
    ) -> String {
        let size = self.size;
        let mut cols_per_row: Vec<Vec<usize>> = vec![vec![]; size];
        for cell in cells {
            cols_per_row[cell.0].push(cell.1 + 1);
        }
        for i in 0..size {
            cols_per_row[i].sort();
        }

        let mut groups: Vec<String> = Vec::new();
        for i in 0..size {
            if cols_per_row[i].len() == 0 {
                continue;
            }

            let mut rows_in_group: Vec<usize> = vec![i + 1];
            for j in i + 1..size {
                if cols_per_row[j] == cols_per_row[i] {
                    rows_in_group.push(j + 1);
                    cols_per_row[j].clear();
                }
            }

            groups.push(format!(
                "r{}c{}",
                rows_in_group.iter().join(cell_separator),
                cols_per_row[i].iter().join(cell_separator)
            ));
        }

        groups.join(&group_separator)
    }

    // Used by compact_name
    fn compact_name_grouped_by_col(
        self,
        cells: &[(usize, usize)],
        cell_separator: &str,
        group_separator: &str,
    ) -> String {
        let size = self.size;
        let mut rows_per_col: Vec<Vec<usize>> = vec![vec![]; size];
        for cell in cells {
            rows_per_col[cell.1].push(cell.0 + 1);
        }
        for i in 0..size {
            rows_per_col[i].sort();
        }

        let mut groups: Vec<String> = Vec::new();
        for i in 0..size {
            if rows_per_col[i].len() == 0 {
                continue;
            }

            let mut cols_in_group: Vec<usize> = vec![i + 1];
            for j in i + 1..size {
                if rows_per_col[j] == rows_per_col[i] {
                    cols_in_group.push(j + 1);
                    rows_per_col[j].clear();
                }
            }

            groups.push(format!(
                "r{}c{}",
                rows_per_col[i].iter().join(&cell_separator),
                cols_in_group.iter().join(&cell_separator)
            ));
        }

        groups.join(&group_separator)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_cell_group() {
        let cu = CellUtility::new(9);
        let cu4 = CellUtility::new(4);
        let cu10 = CellUtility::new(10);

        assert_eq!(cu.parse_cell_groups(""), Result::Ok(vec![]));
        assert_eq!(
            cu.parse_cell_groups("r1c1"),
            Result::Ok(vec![vec![cu.cell(0, 0)]])
        );
        assert_eq!(
            cu.parse_cell_groups("R1C1"),
            Result::Ok(vec![vec![cu.cell(0, 0)]])
        );
        assert_eq!(
            cu.parse_cell_groups("r2c1"),
            Result::Ok(vec![vec![cu.cell(1, 0)]])
        );
        assert_eq!(
            cu4.parse_cell_groups("r2c1"),
            Result::Ok(vec![vec![cu4.cell(1, 0)]])
        );
        assert_eq!(
            cu.parse_cell_groups("r2c2"),
            Result::Ok(vec![vec![cu.cell(1, 1)]])
        );
        assert_eq!(
            cu10.parse_cell_groups("r10c10"),
            Result::Ok(vec![vec![cu10.cell(9, 9)]])
        );
        assert_eq!(
            cu.parse_cell_groups("r1-3c1-2"),
            Result::Ok(vec![vec![
                cu.cell(0, 0),
                cu.cell(0, 1),
                cu.cell(1, 0),
                cu.cell(1, 1),
                cu.cell(2, 0),
                cu.cell(2, 1)
            ]])
        );
        assert_eq!(
            cu.parse_cell_groups("r1c1r2c2"),
            Result::Ok(vec![vec![cu.cell(0, 0), cu.cell(1, 1)]])
        );
        assert_eq!(
            cu.parse_cell_groups("r1c1d222"),
            Result::Ok(vec![vec![
                cu.cell(0, 0),
                cu.cell(1, 0),
                cu.cell(2, 0),
                cu.cell(3, 0)
            ]])
        );
        assert_eq!(
            cu.parse_cell_groups("r1,3c1-2"),
            Result::Ok(vec![vec![
                cu.cell(0, 0),
                cu.cell(0, 1),
                cu.cell(2, 0),
                cu.cell(2, 1)
            ]])
        );
        assert_eq!(
            cu.parse_cell_groups("r1c1;r2c2"),
            Result::Ok(vec![vec![cu.cell(0, 0)], vec![cu.cell(1, 1)]])
        );
        assert!(cu.parse_cell_groups("x").is_err());
        assert!(cu.parse_cell_groups("x1c1").is_err());
        assert!(cu.parse_cell_groups("r0c1").is_err());
        assert!(cu.parse_cell_groups("r2c1d88").is_err());
        assert!(cu.parse_cell_groups("r1-10c1").is_err());
    }

    #[test]
    fn test_cell_names() {
        let cu = CellUtility::new(9);
        assert_eq!(cu.cell_index(0).to_string(), "r1c1");
        assert_eq!(cu.cell_index(40).to_string(), "r5c5");
        assert_eq!(cu.cell_index(80).to_string(), "r9c9");
        assert_eq!(cu.compact_name(&[]), "");
        assert_eq!(cu.compact_name(&[cu.cell(0, 0)]), "r1c1");
        assert_eq!(
            cu.compact_name(&[cu.cell(0, 0), cu.cell(0, 1), cu.cell(0, 2)]),
            "r1c123"
        );
        assert_eq!(
            cu.compact_name(&[cu.cell(0, 0), cu.cell(1, 0), cu.cell(2, 0)]),
            "r123c1"
        );
        assert_eq!(
            cu.compact_name(&[
                cu.cell(0, 0),
                cu.cell(0, 1),
                cu.cell(0, 2),
                cu.cell(1, 0),
                cu.cell(2, 0)
            ]),
            "r123c1,r1c23"
        );
        assert_eq!(
            cu.compact_name(&[cu.cell(0, 0), cu.cell(1, 1), cu.cell(2, 2)]),
            "r1c1,r2c2,r3c3"
        );
        assert_eq!(
            cu.compact_name(&[
                cu.cell(0, 0),
                cu.cell(1, 1),
                cu.cell(2, 2),
                cu.cell(2, 3),
                cu.cell(2, 4)
            ]),
            "r1c1,r2c2,r3c345"
        );
        assert_eq!(
            cu.compact_name(&[
                cu.cell(0, 0),
                cu.cell(1, 1),
                cu.cell(2, 2),
                cu.cell(3, 2),
                cu.cell(4, 2)
            ]),
            "r1c1,r2c2,r345c3"
        );
    }
}
