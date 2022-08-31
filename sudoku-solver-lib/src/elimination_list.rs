use itertools::Itertools;

use crate::{board::Board, board_utility::*, logic_result::LogicResult};
use std::{collections::BTreeSet, fmt::Display};

/// A utility struct for storing a list of eliminated candidates.
///
/// Use `[EliminationList::execute`] to execute the eliminations on a board.
///
/// Use the [`Display`] to get a human-readable description of the list of
/// eliminated candidates.
#[derive(Clone)]
pub struct EliminationList {
    candidates: BTreeSet<usize>,
    board_size: usize,
}

impl EliminationList {
    /// Create a new empty elimination list.
    ///
    /// # Arguments
    /// - `board_size` - The size of the board.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::elimination_list::EliminationList;
    /// // Create an empty elimination list with a board size of 9x9.
    /// let elims = EliminationList::new(9);
    /// ```
    pub fn new(board_size: usize) -> EliminationList {
        EliminationList {
            candidates: BTreeSet::new(),
            board_size,
        }
    }

    /// Add a candidate to the elimination list.
    ///
    /// # Arguments
    /// - `candidate` - The candidate to add to the elimination list.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::elimination_list::EliminationList;
    /// # use sudoku_solver_lib::board_utility::*;
    /// // Create an empty elimination list with a board size of 9x9.
    /// let size = 9;
    /// let mut elims = EliminationList::new(size);
    ///
    /// // Add candidate 3r4c5 to the elimination list.
    ///
    /// // Rows and cols are 0 indexed.
    /// let row = 3;
    /// let col = 4;
    ///
    /// // Values are 1 indexed.
    /// let val = 3;
    ///
    /// // Compute the candidate index
    /// let cell = cell_index(row, col, size);
    /// let candidate = candidate_index(cell, val, size);
    ///
    /// // Add the candidate to the elimination list.
    /// elims.add(candidate);
    ///
    /// // Describe the eliminations
    /// let desc = elims.to_string();
    /// assert_eq!(desc, "-3r4c5");
    /// ```
    pub fn add(&mut self, candidate: usize) {
        self.candidates.insert(candidate);
    }

    /// Add all candidates to the elimination list.
    ///
    /// # Arguments
    /// - `candidates` - The candidates to add to the elimination list.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::elimination_list::EliminationList;
    /// # use sudoku_solver_lib::board_utility::*;
    /// // Create an empty elimination list with a board size of 9x9.
    /// let size = 9;
    /// let mut elims = EliminationList::new(size);
    ///
    /// // Add candidates 1r1c1, 3r4c5, and 3r4c6 to the elimination list.
    /// let candidate1 = candidate_index(cell_index(0, 0, size), 1, size);
    /// let candidate2 = candidate_index(cell_index(3, 4, size), 3, size);
    /// let candidate3 = candidate_index(cell_index(3, 5, size), 3, size);
    /// elims.add_all(&[candidate1, candidate2, candidate3]);
    ///
    /// // Describe the eliminations
    /// let desc = elims.to_string();
    /// assert_eq!(desc, "-1r1c1;-3r4c56");
    /// ```
    /// elims.add_all(&[0, 3 * 9 + 4, 5 * 9 + 8]);
    pub fn add_all(&mut self, candidates: &[usize]) {
        self.candidates.extend(candidates.iter());
    }

    /// Add a candidate to the elimination list by cell index and value.
    ///
    /// # Arguments
    /// - `cell` - The cell index.
    /// - `val` - The value.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::elimination_list::EliminationList;
    /// # use sudoku_solver_lib::board_utility::*;
    /// // Create an empty elimination list with a board size of 9x9.
    /// let size = 9;
    /// let mut elims = EliminationList::new(size);
    ///
    /// // Add candidate 3r4c5 to the elimination list.
    ///
    /// // Rows and cols are 0 indexed.
    /// let row = 3;
    /// let col = 4;
    ///
    /// // Values are 1 indexed.
    /// let val = 3;
    ///
    /// // Compute the candidate index
    /// let cell = cell_index(row, col, size);
    ///
    /// // Add the candidate to the elimination list.
    /// elims.add_cell_value(cell, val);
    ///
    /// // Describe the eliminations
    /// let desc = elims.to_string();
    /// assert_eq!(desc, "-3r4c5");
    /// ```
    pub fn add_cell_value(&mut self, cell: usize, val: usize) {
        self.add(candidate_index(cell, val, self.board_size));
    }

    /// Add a candidate to the elimination list by row, column, and value.
    ///
    /// # Arguments
    /// - `row` - The row index.
    /// - `col` - The column index.
    /// - `val` - The value.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::elimination_list::EliminationList;
    /// # use sudoku_solver_lib::board_utility::*;
    /// // Create an empty elimination list with a board size of 9x9.
    /// let size = 9;
    /// let mut elims = EliminationList::new(size);
    ///
    /// // Add candidate 3r4c5 to the elimination list.
    ///
    /// // Rows and cols are 0 indexed.
    /// let row = 3;
    /// let col = 4;
    ///
    /// // Values are 1 indexed.
    /// let val = 3;
    ///
    /// // Add the candidate to the elimination list.
    /// elims.add_row_col_value(row, col, val);
    ///
    /// // Describe the eliminations
    /// let desc = elims.to_string();
    /// assert_eq!(desc, "-3r4c5");
    /// ```
    pub fn add_row_col_value(&mut self, row: usize, col: usize, val: usize) {
        self.add_cell_value(cell_index(row, col, self.board_size), val);
    }

    /// Remove a candidate from the elimination list.
    /// Returns true if the candidate was removed, false if it was not in the list.
    /// If the candidate was not in the list, this function does nothing.
    pub fn remove(&mut self, candidate: usize) -> bool {
        self.candidates.remove(&candidate)
    }

    /// Get the number of candidates in the elimination list.
    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    /// Get the candidates in the elimination list.
    pub fn candidates(&self) -> &BTreeSet<usize> {
        &self.candidates
    }

    /// Execute the eliminations on a [`Board`].
    ///
    /// # Arguments
    /// - `board` - The [`Board`] to execute the eliminations on.
    ///
    /// # Returns
    /// - [`LogicResult`] - The result of the eliminations.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::elimination_list::EliminationList;
    /// # use sudoku_solver_lib::board_utility::*;
    /// # use sudoku_solver_lib::board::Board;
    /// # use sudoku_solver_lib::logic_result::LogicResult;
    /// // Create a default board.
    /// let mut board = Board::default();
    ///
    /// // Create an empty elimination list.
    /// let size = board.size();
    /// let mut elims = EliminationList::new(size);
    ///
    /// // Add candidates 1r1c1, 3r4c5, and 3r4c6 to the elimination list.
    /// let candidate1 = candidate_index(cell_index(0, 0, size), 1, size);
    /// let candidate2 = candidate_index(cell_index(3, 4, size), 3, size);
    /// let candidate3 = candidate_index(cell_index(3, 5, size), 3, size);
    /// elims.add_all(&[candidate1, candidate2, candidate3]);
    ///
    /// // Perform the eliminations.
    /// let result = elims.execute(&mut board);
    ///
    /// // Check the result.
    /// assert_eq!(result, LogicResult::Changed);
    /// assert!(!board.has_candidate(candidate1));
    /// assert!(!board.has_candidate(candidate2));
    /// assert!(!board.has_candidate(candidate3));
    ///
    /// // Eliminate all candidates from r1c1 - this will make the board invalid.
    /// for val in 1..=9 {
    /// 	let candidate = candidate_index(cell_index(0, 0, size), val, size);
    /// 	elims.add(candidate);
    /// }
    /// let result = elims.execute(&mut board);
    /// assert_eq!(result, LogicResult::Invalid);
    /// ```
    pub fn execute(&self, board: &mut Board) -> LogicResult {
        let mut result = LogicResult::None;
        for candidate in self.candidates.iter() {
            if board.has_candidate(*candidate) {
                if board.clear_candidate(*candidate) {
                    if result == LogicResult::None {
                        result = LogicResult::Changed;
                    }
                } else {
                    result = LogicResult::Invalid;
                }
            }
        }

        result
    }
}

impl Display for EliminationList {
    /// Display the elimination list.
    /// The format is a semi-colon-separated list of candidates.
    ///
    /// # Examples
    /// - `1r1c1, 1r1c2, 1r1c3`: `"-1r1c123"`
    /// - `1r1c1, 2r1c1, 2r2c1`: `"-1r1c1;-2r12c1"`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut elims_by_value: Vec<Vec<usize>> = vec![vec![]; self.board_size];
        for &candidate in self.candidates.iter() {
            let (cell, value) = candidate_index_to_cell_and_value(candidate, self.board_size);
            elims_by_value[value - 1].push(cell);
        }

        let mut descs: Vec<String> = Vec::new();
        for val in 1..=self.board_size {
            if !elims_by_value[val - 1].is_empty() {
                elims_by_value[val - 1].sort();
                let cur_desc = format!(
                    "-{}{}",
                    val,
                    compact_name(&elims_by_value[val - 1], self.board_size)
                );
                descs.push(cur_desc);
            }
        }

        let desc = descs.iter().join(";");
        write!(f, "{}", desc)
    }
}
