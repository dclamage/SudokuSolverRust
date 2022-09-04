//! Constains the [`Solver`] struct which is the main entry point for solving a puzzle.

use itertools::Itertools;

use crate::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct Solver {
    board: Board,
    logical_solve_steps: Vec<Arc<dyn LogicalStep>>,
    brute_force_steps: Vec<Arc<dyn LogicalStep>>,
}

impl Solver {
    /// Create a new solver.
    ///
    /// # Arguments
    /// * `size` - The size of the board (use 9 for a 9x9 board).
    /// * `regions` - The regions of the board. Pass an empty slice to use default regions.
    /// * `logical_steps` - The logical steps that should be used to solve the puzzle.
    /// Pass an empty slice to use default logical steps.
    /// * `constraints` - The additional constraints that should be used to solve the puzzle, if any.
    pub fn new(
        size: usize,
        regions: &[usize],
        logical_steps: impl Iterator<Item = Arc<dyn LogicalStep>>,
        constraints: impl Iterator<Item = Arc<dyn Constraint>>,
    ) -> Solver {
        let constraints: Vec<_> = constraints.collect();
        let board = Board::new(size, regions, &constraints);
        let mut logical_steps: Vec<_> = logical_steps.collect();
        if logical_steps.is_empty() {
            logical_steps = Self::standard_logic();
        } else {
            // Ensure all the required logical steps are present in the list.
            // Required steps generally happen first, so they're added to the
            // front.
            for required_logic in Self::required_logic() {
                if !logical_steps
                    .iter()
                    .any(|l| l.name() == required_logic.name())
                {
                    let required_logic_slice = [required_logic];
                    logical_steps.splice(..0, required_logic_slice.iter().cloned());
                }
            }
        }

        let logical_solve_steps = logical_steps
            .iter()
            .cloned()
            .filter(|step| step.is_active_during_logical_solves())
            .collect();

        let brute_force_steps = logical_steps
            .iter()
            .cloned()
            .filter(|step| step.is_active_during_brute_force_solves())
            .collect();

        Solver {
            board,
            logical_solve_steps,
            brute_force_steps,
        }
    }

    fn required_logic() -> Vec<Arc<dyn LogicalStep>> {
        vec![Arc::new(AllNakedSingles), Arc::new(NakedSingle)]
    }

    pub fn standard_logic() -> Vec<Arc<dyn LogicalStep>> {
        vec![
            Arc::new(AllNakedSingles),
            Arc::new(HiddenSingle),
            Arc::new(NakedSingle),
            Arc::new(SimpleCellForcing),
        ]
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    pub fn logical_solve_steps(&self) -> &[Arc<dyn LogicalStep>] {
        &self.logical_solve_steps
    }

    pub fn brute_force_steps(&self) -> &[Arc<dyn LogicalStep>] {
        &self.brute_force_steps
    }

    pub fn set_givens(&mut self, givens: impl Iterator<Item = (CellIndex, usize)>) -> LogicResult {
        let mut changed = false;

        for (cell, value) in givens {
            if !self.board.cell(cell).is_solved() {
                if !self.board.set_solved(cell, value) {
                    return LogicResult::Invalid;
                }
                changed = true;
            }
        }

        if changed {
            LogicResult::Changed
        } else {
            LogicResult::None
        }
    }

    /// Convert set the givens from a given string.
    /// The string should be a sequence of numbers, with 0 or any non-digit representing an empty cell.
    /// The string should be in row-major order.
    /// For grid sizes larger than 9, the each number takes the same number of characters, so use 01 for 1, for example.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::prelude::*;
    /// let mut solver = Solver::default();
    /// let result = solver.set_givens_from_string("123000000000000000000000000000000000000000000000000000000000000000000000000000000");
    /// assert!(result == LogicResult::Changed);
    ///
    /// let cu = solver.board().cell_utility();
    /// assert!(solver.board().cell(cu.cell(0, 0)).is_solved());
    /// assert!(solver.board().cell(cu.cell(0, 1)).is_solved());
    /// assert!(solver.board().cell(cu.cell(0, 2)).is_solved());
    /// assert!(!solver.board().cell(cu.cell(0, 3)).is_solved());
    /// assert_eq!(solver.board().cell(cu.cell(0, 0)).value(), 1);
    /// assert_eq!(solver.board().cell(cu.cell(0, 1)).value(), 2);
    /// assert_eq!(solver.board().cell(cu.cell(0, 2)).value(), 3);
    /// assert_eq!(solver.board().cell(cu.cell(0, 3)).min(), 4);
    ///
    /// let mut solver16 = Solver::new(16, &[], std::iter::empty(), std::iter::empty());
    /// ```
    pub fn set_givens_from_string(&mut self, givens: &str) -> LogicResult {
        let cu = self.board.cell_utility();
        if cu.size() <= 9 {
            if givens.len() != cu.size() * cu.size() {
                return LogicResult::Invalid;
            }

            let givens_itr = givens.chars().enumerate().filter_map(|(i, c)| {
                let value = c.to_digit(10)?;
                if value == 0 {
                    None
                } else {
                    Some((cu.cell_index(i), value as usize))
                }
            });
            self.set_givens(givens_itr)
        } else {
            let num_digits = cu.size().to_string().len();
            if givens.len() != cu.size() * cu.size() * num_digits {
                return LogicResult::Invalid;
            }

            let givens_chunks_itr = givens.chars().chunks(num_digits);
            let givens_itr = givens_chunks_itr
                .into_iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    // Convert the chunk into a string.
                    let val_str = c.collect::<String>();

                    // Convert the string into a number.
                    let value = val_str.parse::<usize>().ok()?;

                    // If the value is 0, ignore it.
                    if value == 0 {
                        None
                    } else {
                        Some((cu.cell_index(i), value))
                    }
                });
            self.set_givens(givens_itr)
        }
    }
}

impl Default for Solver {
    fn default() -> Self {
        Solver::new(9, &[], std::iter::empty(), std::iter::empty())
    }
}
