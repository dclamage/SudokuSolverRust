//! Constains the [`Solver`] struct which is the main entry point for solving a puzzle.

pub mod logical_solve_result;
pub mod prelude;
pub mod single_solution_result;
pub mod solver_builder;

use crate::prelude::*;
use std::sync::Arc;

/// The main entry point for solving a puzzle.
///
/// Use the [`SolverBuilder`] struct to create a [`Solver`].
/// Do not create a [`Solver`] directly.
///
/// The [`Solver`] struct contains a [`Board`] which represents the current state of the puzzle to be solved.
///
/// The logic of the solver can be expanded using the [`LogicalStep`] trait.
/// This libary contains some basic implementations of this trait for core functionality.
/// Additional implementations can be added by the consumer of this library
/// to logically solve more complex puzzles.
///
/// Additionally, the [`Solver`] struct contains a list of [`Constraint`]s which define the rules of the puzzle.
/// This library does not provide any implementations of this trait, and instead relies on the
/// consumer of this library to provide the constraints for the puzzle to be solved.
#[derive(Clone)]
pub struct Solver {
    board: Board,
    logical_solve_steps: Vec<Arc<dyn LogicalStep>>,
    brute_force_steps: Vec<Arc<dyn LogicalStep>>,
}

impl Solver {
    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn size(&self) -> usize {
        self.board.size()
    }

    pub fn cell_utility(&self) -> CellUtility {
        self.board.cell_utility()
    }

    /// Find a single logical step that can be applied to the puzzle.
    pub fn run_single_logical_step(&mut self) -> LogicalStepResult {
        for step in self.logical_solve_steps.iter() {
            let step_result = step.run(&mut self.board, true);
            if !step_result.is_none() {
                if step.has_own_prefix() {
                    return step_result;
                } else {
                    return step_result.with_prefix(format!("{}: ", step.name()).as_str());
                }
            }
        }

        LogicalStepResult::None
    }

    /// Run a full logical solve. This mutates the solver's board.
    pub fn run_logical_solve(&mut self) -> LogicalSolveResult {
        let mut desc_list = LogicalStepDescList::new();
        let mut changed = false;
        loop {
            if self.board.is_solved() {
                desc_list.push("Solved!".into());
                return LogicalSolveResult::Solved(desc_list);
            }

            let step_result = self.run_single_logical_step();
            if step_result.is_none() {
                break;
            }

            changed = true;

            if let Some(desc) = step_result.description() {
                desc_list.push(desc.clone());
            }

            if step_result.is_invalid() {
                return LogicalSolveResult::Invalid(desc_list);
            }
        }

        if changed {
            LogicalSolveResult::Changed(desc_list)
        } else {
            LogicalSolveResult::None
        }
    }

    fn run_single_brute_force_step(&self, board: &mut Board) -> LogicalStepResult {
        for step in self.brute_force_steps.iter() {
            let step_result = step.run(board, false);
            if !step_result.is_none() {
                return step_result;
            }
        }

        LogicalStepResult::None
    }

    fn run_brute_force_logic(&self, board: &mut Board) -> bool {
        loop {
            let step_result = self.run_single_brute_force_step(board);
            if step_result.is_none() {
                break;
            }
            if step_result.is_invalid() {
                return false;
            }
        }

        true
    }

    /// Use brute-force methods to find the first solution to the puzzle.
    ///
    /// The solution is the lexicographically first solution and is not
    /// guaranteed to be the only solution.
    pub fn find_first_solution(&self) -> SingleSolutionResult {
        let cu = self.cell_utility();
        let mut board_stack = Vec::new();
        board_stack.push((Box::new(self.board.clone()), cu.cell(0, 0)));

        while !board_stack.is_empty() {
            let (mut board, mut cell) = board_stack.pop().unwrap();
            if !self.run_brute_force_logic(&mut board) {
                continue;
            }

            if board.is_solved() {
                return SingleSolutionResult::Solved(board);
            }

            loop {
                if board.cell(cell).is_solved() {
                    if let Some(next_cell) = cell.next_cell() {
                        cell = next_cell;
                    } else {
                        break;
                    }
                } else {
                    let mask = board.cell(cell);
                    let value = mask.min();

                    // Push a copy of the board onto the stack with the value unset.
                    let mut board_copy = board.clone();
                    if board_copy.clear_value(cell, value) {
                        board_stack.push((board_copy, cell));
                    }

                    // Push a the board onto the stack with the value solved.
                    if board.set_solved(cell, value) {
                        board_stack.push((board, cell));
                    }

                    break;
                }
            }
        }

        SingleSolutionResult::None
    }

    fn find_best_brute_force_cell(board: &Board) -> Option<CellIndex> {
        let mut best_cell = None;
        let mut best_cell_candidate_count = usize::MAX;
        let board_data = board.data();

        for &cell in board_data.powerful_cells() {
            let mask = board.cell(cell);
            if mask.is_solved() {
                continue;
            }

            let cell_count = mask.count();
            if cell_count <= 2 {
                return Some(cell);
            }
            if cell_count < best_cell_candidate_count {
                best_cell = Some(cell);
                best_cell_candidate_count = cell_count;
            }
        }

        if best_cell.is_some() {
            return best_cell;
        }

        for cell in board.all_cells() {
            let mask = board.cell(cell);
            if mask.is_solved() {
                continue;
            }

            let cell_count = mask.count();
            if cell_count == 1 {
                continue;
            }

            if cell_count == 2 {
                return Some(cell);
            }

            if cell_count < best_cell_candidate_count {
                best_cell = Some(cell);
                best_cell_candidate_count = cell_count;
            }
        }

        best_cell
    }

    /// Use brute-force methods to find a random solution to the puzzle.
    /// This can be faster than [`Solver::find_first_solution`] because it
    /// is not forced to find the lexicographically first solution.
    ///
    /// The solution is not guaranteed to be the only solution.
    pub fn find_random_solution(&self) -> SingleSolutionResult {
        let mut board_stack = Vec::new();
        board_stack.push(Box::new(self.board.clone()));

        while !board_stack.is_empty() {
            let mut board = board_stack.pop().unwrap();
            if !self.run_brute_force_logic(&mut board) {
                continue;
            }

            if board.is_solved() {
                return SingleSolutionResult::Solved(board);
            }

            let cell = Self::find_best_brute_force_cell(&board);
            if let Some(cell) = cell {
                let mask = board.cell(cell);
                let value = mask.random();

                // Push a copy of the board onto the stack with the value unset.
                let mut board_copy = board.clone();
                if board_copy.clear_value(cell, value) {
                    board_stack.push(board_copy);
                }

                // Push a the board onto the stack with the value solved.
                if board.set_solved(cell, value) {
                    board_stack.push(board);
                }
            } else {
                return SingleSolutionResult::Error(
                    "Internal error finding a cell to check.".to_owned(),
                );
            }
        }

        SingleSolutionResult::None
    }
}

impl Default for Solver {
    fn default() -> Self {
        SolverBuilder::new(9).build().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_first_solution() {
        let solver = Solver::default();

        let result = solver.find_first_solution();
        assert!(result.is_solved());

        let board = result.board().unwrap();
        assert!(board.is_solved());

        let solution = board.to_string();
        assert_eq!(
            solution,
            "123456789456789123789123456214365897365897214897214365531642978642978531978531642"
        );
    }

    #[test]
    fn test_random_solution() {
        let solver = Solver::default();

        let result = solver.find_random_solution();
        assert!(result.is_solved());

        let board = result.board().unwrap();
        assert!(board.is_solved());

        let solution = board.to_string();
        assert!(solution.len() == 81);
        assert!(!solution.chars().any(|c| c < '1' || c > '9'));
    }
}
