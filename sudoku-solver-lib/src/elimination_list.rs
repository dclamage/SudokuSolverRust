//! Contains [`EliminationList`] for storing a list of eliminated candidates.

use crate::prelude::*;
use itertools::Itertools;
use std::{collections::BTreeSet, fmt::Display};

/// A utility struct for storing a list of eliminated candidates.
///
/// Use `[EliminationList::execute`] to execute the eliminations on a board.
///
/// Use the [`Display`] to get a human-readable description of the list of
/// eliminated candidates.
#[derive(Clone)]
pub struct EliminationList {
    candidates: BTreeSet<CandidateIndex>,
}

impl EliminationList {
    /// Create a new empty elimination list.
    pub fn new() -> EliminationList {
        EliminationList {
            candidates: BTreeSet::new(),
        }
    }

    /// Get the number of candidates in the elimination list.
    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    /// Get if the elimination list is empty.
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    /// Get the candidates in the elimination list.
    pub fn candidates(&self) -> &BTreeSet<CandidateIndex> {
        &self.candidates
    }

    /// Returns true if the list contains the given candidate.
    pub fn contains(&self, candidate: CandidateIndex) -> bool {
        self.candidates.contains(&candidate)
    }

    /// Returns an iterator over the candidates in the list.
    pub fn iter(&self) -> impl Iterator<Item = CandidateIndex> + '_ {
        self.candidates.iter().copied()
    }

    /// Add a candidate to the elimination list.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::elimination_list::EliminationList;
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// // Create an empty elimination list with a board size of 9x9.
    /// let cu = CellUtility::new(9);
    /// let mut elims = EliminationList::new();
    ///
    /// // Add candidate 3r4c5 to the elimination list.
    /// let candidate = cu.candidate(cu.cell(3, 4), 3);
    ///
    /// // Add the candidate to the elimination list.
    /// elims.add(candidate);
    ///
    /// // Describe the eliminations
    /// let desc = elims.to_string();
    /// assert_eq!(desc, "-3r4c5");
    /// ```
    pub fn add(&mut self, candidate: CandidateIndex) {
        self.candidates.insert(candidate);
    }

    /// Add all candidates to the elimination list.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::elimination_list::EliminationList;
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// // Create an empty elimination list with a board size of 9x9.
    /// let cu = CellUtility::new(9);
    /// let mut elims = EliminationList::new();
    ///
    /// // Add candidates 1r1c1, 3r4c5, and 3r4c6 to the elimination list.
    /// let candidate1 = cu.cell(0, 0).candidate(1);
    /// let candidate2 = cu.cell(3, 4).candidate(3);
    /// let candidate3 = cu.cell(3, 5).candidate(3);
    /// elims.add_all(&[candidate1, candidate2, candidate3]);
    ///
    /// // Describe the eliminations
    /// let desc = elims.to_string();
    /// assert_eq!(desc, "-1r1c1;-3r4c56");
    /// ```
    pub fn add_all(&mut self, candidates: &[CandidateIndex]) {
        self.candidates.extend(candidates.iter());
    }

    /// Add a candidate to the elimination list by cell index and value.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::elimination_list::EliminationList;
    /// # use sudoku_solver_lib::cell_utility::CellUtility;
    /// // Create an empty elimination list with a board size of 9x9.
    /// let cu = CellUtility::new(9);
    /// let mut elims = EliminationList::new();
    ///
    /// // Add the candidate to the elimination list.
    /// elims.add_cell_value(cu.cell(3, 4), 3);
    ///
    /// // Describe the eliminations
    /// let desc = elims.to_string();
    /// assert_eq!(desc, "-3r4c5");
    /// ```
    pub fn add_cell_value(&mut self, cell: CellIndex, value: usize) {
        self.add(cell.candidate(value));
    }

    /// Remove a candidate from the elimination list.
    /// Returns true if the candidate was removed, false if it was not in the list.
    /// If the candidate was not in the list, this function does nothing.
    pub fn remove(&mut self, candidate: CandidateIndex) -> bool {
        self.candidates.remove(&candidate)
    }

    /// Execute the eliminations on a [`Board`].
    ///
    /// # Returns
    /// - [`LogicalStepResult`] - The result of the eliminations.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::prelude::*;
    /// // Create a default board.
    /// let mut board = Board::default();
    ///
    /// // Create an empty elimination list.
    /// let size = board.size();
    /// let cu = CellUtility::new(size);
    /// let mut elims = EliminationList::new();
    ///
    /// // Add candidates 1r1c1, 3r4c5, and 3r4c6 to the elimination list.
    /// let candidate1 = cu.cell(0, 0).candidate(1);
    /// let candidate2 = cu.cell(3, 4).candidate(3);
    /// let candidate3 = cu.cell(3, 5).candidate(3);
    /// elims.add_all(&[candidate1, candidate2, candidate3]);
    ///
    /// // Perform the eliminations.
    /// let result = elims.execute(&mut board);
    ///
    /// // Check the result.
    /// assert!(result.is_changed());
    /// assert!(!board.has_candidate(candidate1));
    /// assert!(!board.has_candidate(candidate2));
    /// assert!(!board.has_candidate(candidate3));
    ///
    /// // Eliminate all candidates from r1c1 - this will make the board invalid.
    /// for val in 1..=9 {
    ///     let candidate = cu.cell(0, 0).candidate(val);
    ///     elims.add(candidate);
    /// }
    /// let result = elims.execute(&mut board);
    /// assert!(result.is_invalid());
    /// ```
    pub fn execute(&self, board: &mut Board) -> LogicalStepResult {
        let mut result = LogicalStepResult::None;
        for &candidate in self.candidates.iter() {
            if board.has_candidate(candidate) {
                if board.clear_candidate(candidate) {
                    if result.is_none() {
                        result = LogicalStepResult::Changed(None);
                    }
                } else {
                    return LogicalStepResult::Invalid(None);
                }
            }
        }

        result
    }
}

impl Default for EliminationList {
    fn default() -> Self {
        Self::new()
    }
}

impl From<BTreeSet<CandidateIndex>> for EliminationList {
    fn from(candidates: BTreeSet<CandidateIndex>) -> Self {
        Self { candidates }
    }
}

impl From<EliminationList> for BTreeSet<CandidateIndex> {
    fn from(elims: EliminationList) -> Self {
        elims.candidates
    }
}

impl FromIterator<CandidateIndex> for EliminationList {
    /// Create an elimination list from an iterator of candidates.
    fn from_iter<I>(iter: I) -> EliminationList
    where
        I: IntoIterator<Item = CandidateIndex>,
    {
        EliminationList {
            candidates: iter.into_iter().collect(),
        }
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
        if self.candidates.is_empty() {
            write!(f, "No eliminations")
        } else {
            let board_size = self.candidates.iter().next().unwrap().size();
            let cu = CellUtility::new(board_size);
            let mut elims_by_value: Vec<Vec<CellIndex>> = vec![vec![]; board_size];
            for &candidate in self.candidates.iter() {
                let (cell, value) = candidate.cell_index_and_value();
                elims_by_value[value - 1].push(cell);
            }

            let mut descs: Vec<String> = Vec::new();
            for val in 1..=board_size {
                if !elims_by_value[val - 1].is_empty() {
                    elims_by_value[val - 1].sort();
                    let cur_desc = format!("-{}{}", val, cu.compact_name(&elims_by_value[val - 1]));
                    descs.push(cur_desc);
                }
            }

            let desc = descs.iter().join(";");
            write!(f, "{}", desc)
        }
    }
}
